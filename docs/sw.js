// Service worker: makes the game installable and playable offline.
//
// Everything here is relative. GitHub Pages serves this project from a
// subpath (/bevy-puzzle-game/), so an absolute "/index.html" would point at
// the root of the domain and fetch somebody else's page.
//
// The wasm bundle is ~21 MB. That is the whole reason a worker earns its place
// here: without one the browser refetches it on every visit, and with one the
// second launch is instant. It is also why the install step does **not**
// precache it — a 21 MB download blocking activation would make the first
// visit slower, which is the visit that matters most. The shell is precached,
// the bundle is cached the first time it is actually fetched.

const VERSION = 'v1'
const SHELL = `shell-${VERSION}`
const RUNTIME = `runtime-${VERSION}`

// Small, and needed before anything can draw.
const SHELL_FILES = [
  './',
  './index.html',
  './manifest.webmanifest',
  './icons/icon-192.png',
  './icons/icon-512.png',
  './icons/icon-maskable-512.png',
]

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches
      .open(SHELL)
      // Individually, not addAll: addAll rejects the whole batch if any one
      // file 404s, which would leave the worker permanently uninstalled over a
      // single missing icon.
      .then((cache) =>
        Promise.all(
          SHELL_FILES.map((file) => cache.add(file).catch(() => undefined))
        )
      )
      .then(() => self.skipWaiting())
  )
})

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((names) =>
        Promise.all(
          names
            .filter((name) => name !== SHELL && name !== RUNTIME)
            .map((name) => caches.delete(name))
        )
      )
      .then(() => self.clients.claim())
  )
})

self.addEventListener('fetch', (event) => {
  const request = event.request

  // Only GETs, and only our own origin. A cross-origin request cached here
  // would be an opaque response of unknown size counting against quota.
  if (request.method !== 'GET' || new URL(request.url).origin !== self.location.origin) {
    return
  }

  // Navigations go to the network first so a deploy is picked up on the next
  // visit rather than being pinned to whatever was cached, and fall back to the
  // shell when offline.
  if (request.mode === 'navigate') {
    event.respondWith(
      fetch(request)
        .then((response) => {
          const copy = response.clone()
          caches.open(SHELL).then((cache) => cache.put('./index.html', copy))
          return response
        })
        .catch(() => caches.match('./index.html', { ignoreSearch: true }))
    )
    return
  }

  // Everything else — the wasm, the JS glue, the font, the sounds — is
  // cache-first. These are the big, immutable files; a build that changes them
  // changes VERSION, and the old cache is dropped on activate.
  event.respondWith(
    caches.match(request).then((hit) => {
      if (hit) return hit

      return fetch(request).then((response) => {
        // Only cache what actually arrived intact. A partial (206) or an error
        // stored here would be served back forever.
        if (response && response.status === 200 && response.type === 'basic') {
          const copy = response.clone()
          caches.open(RUNTIME).then((cache) => cache.put(request, copy))
        }
        return response
      })
    })
  )
})
