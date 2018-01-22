let $ = s => document.querySelector(s);
let worker = new Worker("worker.js");
let nextId = 1;

$("#files").addEventListener("change", event => {
  let files = event.target.files;
  for (let i = 0; i < files.length; i++) {
    addFile(files.item(i));
  }
});

function addFile(file) {
  let id = nextId++;
  let template = document.querySelector("#file");
  let content = document.importNode(template, true).content;
  content.querySelector(".file").id = `file-${id}`;
  content.querySelector(".name").textContent = file.name;
  content.querySelector(".save").download = `${file.name}.srt`;
  content.querySelector(".close").addEventListener("click", event => {
    event.preventDefault();
    $(`#file-${id}`).outerHTML = "";
  });
  $("#list").appendChild(content);

  if (file.size > 100 * 1024 * 1024)
    return onConvertError(id, "file too large (> 100 MiB)");
  let inCharset = $("#in-charset").value || null;
  let outCharset = $("#out-charset").value || null;
  let chineseConv = $("#chinese-conv").value || null;
  worker.postMessage({
    id: id, file: file,
    inCharset: inCharset, outCharset: outCharset,
    chineseConv: chineseConv
  });
}

function onConvertError(id, msg) {
  let content = $(`#file-${id}`);
  content.classList.remove("progress");
  content.classList.add("error");
  content.querySelector(".status").textContent = msg;
}

function onConvertDone(id, url) {
  let content = $(`#file-${id}`);
  content.classList.remove("progress");
  content.classList.add("done");
  content.querySelector(".save").href = url;
  content.querySelector(".close").addEventListener("click", event => {
    URL.revokeObjectURL(url);
  });
}

worker.onmessage = function(e) {
  let result = e.data;
  if (result.error) {
    onConvertError(result.id, result.error);
  } else {
    onConvertDone(result.id, result.srtUrl);
  }
}
