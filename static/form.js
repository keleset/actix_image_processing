//Save request:
function sendForm() {
  let form = {urls: []};
  document.getElementsByName("urls").forEach((elem) => { form.urls.push(elem.value)});
  let xhr = new XMLHttpRequest();
  xhr.onreadystatechange = function() {
    if (xhr.readyState == XMLHttpRequest.DONE) {
        alert(xhr.responseText);
    }
  }
  let params = JSON.stringify(form);
  xhr.open('POST', '/upload/remote', true);
  xhr.setRequestHeader("Content-type", "application/json");  
  xhr.send(params);
}
