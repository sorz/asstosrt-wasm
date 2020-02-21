let $ = s => document.querySelector(s);
let worker = new Worker("worker.js");
let nextId = 1;

$("#files").addEventListener("change", async ev => {
  let files = [];
  for (let i = 0; i < ev.target.files.length; i++) {
    files.push(ev.target.files.item(i));
  }
  submit(files);
});

$("#like").addEventListener("click", function(ev) {
  this.classList.add('liked');
  this.title = "Thanks!";
});

const preloadDict = (d) => worker.postMessage({
  action: "preloadDict", dict: d,
});
preloadDict($("#conv-dict").value);
$("#conv-dict").addEventListener("change", ev =>
  preloadDict(ev.target.value));

function onDrop(ev) {
  ev.preventDefault();
  let items = ev.dataTransfer.items
    .filter(i => i.kind == "file")
    .map(f => f.getAsFile());
  submit(items);
};

function onDropOver(ev) {
  ev.preventDefault();
};

function onDropEnd(ev) {
  ev.dataTransfer.clearData();
}

function submit(files) {
  if ($("#no-zip").checked)
    files.forEach(f => addFiles([f]));
  else
    addFiles(files);
}

function addFiles(files) {
  let id = nextId++;
  let template = document.querySelector("#file");
  let content = document.importNode(template, true).content;
  content.querySelector(".file").id = `file-${id}`;
  if (files.length == 1) {
    let name = files[0].name;
    if (name.match(/\.(ass|ssa)$/) != null)
      name = name.slice(0, -4);
    name += '.srt'
    content.querySelector(".name").textContent = name;
    content.querySelector(".save").download = name;
  } else {
    content.querySelector(".name").textContent =
      `${files.length} subtitle files`;
    content.querySelector(".save").download = `srt_subtitles.zip`;
  }
  content.querySelector(".close").addEventListener("click", event => {
    event.preventDefault();
    $(`#file-${id}`).outerHTML = "";
  });
  $("#list").appendChild(content);

  let size = files.reduce((n, f) => n + f.size, 0);
  if (size > 100 * 1024 * 1024)
    return onConvertError(id, "files too large (> 100 MiB)");

  let opts = {
    in_charset: $("#in-charset").value || null,
    out_charset: $("#out-charset").value || null,
    lines: $("#lines").value,
    ignore_codec_err: $("#ignore-codec-err").checked,
    offset_secs: parseFloat($("#offset").value) || 0,
  };
  let cmd = { id: id, opts: opts };
  if (files.length == 1) {
    cmd.action = "addFile";
    cmd.file = files[0];
  } else {
    cmd.action = "addFiles";
    cmd.files = files;
  }
  worker.postMessage(cmd);
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
  $('#vote').style.display = 'block';
}

worker.onmessage = function(e) {
  let result = e.data;
  if (result.error) {
    onConvertError(result.id, result.error);
  } else {
    onConvertDone(result.id, result.url);
  }
}
