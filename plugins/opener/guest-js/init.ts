// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from '@tauri-apps/api/core'

// open <a href="..."> links with the API
window.addEventListener('click', function (evt) {
  if (evt.defaultPrevented || evt.button !== 0 || evt.metaKey || evt.altKey)
    return

  const a = evt
    .composedPath()
    .find((el) => el instanceof Node && el.nodeName.toUpperCase() === 'A') as
    | HTMLAnchorElement
    | undefined

  if (
    !a ||
    !a.href ||
    // only open if supposed to be open in a new tab
    !(a.target === '_blank' || evt.ctrlKey || evt.shiftKey)
  )
    return

  const url = new URL(a.href)

  if (
    // only open if not same origin
    url.origin === window.location.origin ||
    // only open default protocols
    ['http:', 'https:', 'mailto:', 'tel:'].every((p) => url.protocol !== p)
  )
    return

  evt.preventDefault()

  void invoke('plugin:opener|open_url', {
    path: url
  })
})
