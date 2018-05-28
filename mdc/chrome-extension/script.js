'use strict'

var XMLHttpRequest, chrome, alert

function openLink (info, tab) {
  console.log(info.pageUrl, info.linkUrl)
  let xhr = new XMLHttpRequest()

  xhr.onload = () => {
    console.log('xhr success')
  }
  xhr.onerror = () => {
    console.log('xhr failure')
    alert('xhr failure')
  }
  xhr.open('POST', 'http://localhost:9922/enqueue')
  xhr.setRequestHeader("Content-Type", "application/json;charset=UTF-8")
  xhr.send(JSON.stringify({
    uris: [info.linkUrl]
  }))
  xhr.send()
}

chrome.contextMenus.create({
  title: 'Send to mediad',
  contexts: ['link'],
  onclick: openLink
})
