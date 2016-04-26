'use strict'

var XMLHttpRequest, chrome, alert

function getword (info, tab) {
  console.log(info.pageUrl, info.linkUrl)
  let xhr = new XMLHttpRequest()

  xhr.onload = () => {
    console.log('xhr success')
  }
  xhr.onerror = () => {
    console.log('xhr failure')
    alert('xhr failure')
  }
  xhr.open('POST', 'http://localhost:9922/submit', true)
  // xhr.setRequestHeader("Content-Type", "application/json;charset=UTF-8")
  // xhr.send(JSON.stringify({link: info.linkUrl}))
  xhr.send(info.linkUrl)
}

chrome.contextMenus.create({
  title: 'Open link in MPV',
  contexts: ['link'],
  onclick: getword
})
