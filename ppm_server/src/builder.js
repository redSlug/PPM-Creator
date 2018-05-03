const d = document;
let color_data = [];
const rows = 16;
const cols = 16;
console.log('hi')
function build_base () {
  let grid = d.querySelector(".grid")
  let table = "";
  for (let i = 0; i < rows; i++) {
    let row_data = [];
    let tr = d.createElement("tr");
    tr.id = `row-${i}`;
    for (let j = 0; j < cols; j++) {
      let td = d.createElement("td");
      let color = "#000000";
      let color_picker = d.createElement("input");
      td.id = `cell-${i}-${j}`;
      row_data.push(color);
      color_picker.setAttribute("type", "color");
      color_picker.dataset.i = i;
      color_picker.dataset.j = j;
      color_picker.value = color;
      td.appendChild(color_picker);
      tr.appendChild(td);
    }
    grid.appendChild(tr);
    color_data.push(row_data);
  }
}

function newColor(e) {
    let i = e.target.dataset.i;
    let j = e.target.dataset.j;
    color_data[i][j] = e.target.value;
    console.log(color_data);
}

function submitMatrix(e) {
  e.preventDefault()
  let matrix_data = [].concat.apply([], color_data).join("");
  console.log(matrix_data);
  let url = "http://127.0.0.1:8080";
  let body_data = JSON.stringify({"data": `cols=${cols}${matrix_data}`});
  var request_data = {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "Access-Control-Allow-Origin": "*"
    },
    cache: "no-cache",
    body: body_data,
    mode: "cors",
  };
  var request = new Request(url);
  fetch(request, request_data);
}

(function setup() {
  build_base();
  d.querySelectorAll("input").forEach(input => input.addEventListener('change', newColor));
  const button = d.querySelector("#submit")
  button.addEventListener('click', submitMatrix);
})()
