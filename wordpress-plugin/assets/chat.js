var c = {
  "dev":
  {
    "WP_SERVER": "https://wp.sophoservices.com/get_username.php",
    "CHAT_SERVER": "ws://localhost:9001"
  },
  "stage":
  {
    "WP_SERVER": "https://wp.sophoservices.com/get_username.php",
    "CHAT_SERVER": "ws://chat.sophoservices.com:9001"
  }
}
function update_config() {
  var mode = getEl("#mode_select").value;
  WP_SERVER = c[mode]['WP_SERVER'];
  CHAT_SERVER = c[mode]['CHAT_SERVER'];
}
var WP_SERVER = c['dev']['WP_SERVER'];
var CHAT_SERVER = c['dev']['CHAT_SERVER'];

function log(msg) {
  var log = document.querySelector('#log');
  log.innerText = log.innerText + msg + "\n";
}
function getEl(f) {
  return document.querySelector(f);
}
function getField(f) {
  return document.querySelector(f).value;
}
function updateField(f, t) {
  document.querySelector(f).innerText = t;
}
function updateFieldHTML(f, t) {
  document.querySelector(f).innerHTML = t;
}
function show(f) {
  document.querySelector(f).style.display = "block";
}
function hide(f) {
  document.querySelector(f).style.display = "none";
}
function parseStat(stats) {
  try {
    x = JSON.parse(stats);
    updateField("#users", x.users);
  } catch (err) {
    console.log("invalid data from server: " + err);
  }
}
function parseUserlist(userlist) {
  try {
    var list = "";
    x = JSON.parse(userlist);
    console.log(x);
    for (const u in x.users) {
      list += "<div>" + x.users[u].user + "</div>";
      console.log(u);
    }
    updateFieldHTML("#userlist", list);
    show("#userlist-div");
  } catch (err) {
    console.log("invalid data from server - userlist: " + err);
  }
}
function getUserList() {
  ws.send("/USERS");
}
function popup_message(msg) {
  updateFieldHTML("#message-text", msg);
  show("#msg-div");
}
var ws;

function letsgo() {
  fetch(WP_SERVER, { credentials: "include", cache: "no-store" })
    .then(function (r) {
      if (!r.ok) { throw new Error(`HTTP error! status: ${r.status}`); }
      return r.text();
    })
    .then(function (t) {
      connect(t);
    });
}

function connect(info) {
  ws = new WebSocket(CHAT_SERVER);
  var ws_connected = false;
  ws.onopen = function () {
    log('connected');
    hide('#connect');
    show('#interact');
    ws_connected = true;
    ws.send("/USER " + info);
  };
  ws.onclose = function (ev) {
    log('closed');
    show('#connect');
    hide('#interact');
    ws_connected = false;
  };
  ws.onmessage = function (ev) {
    var d = ev.data;
    if (d.startsWith('!*STAT ')) {
      // information
      parseStat(d.substring(7)); // after '!*STAT '
    } else if (d.startsWith('!*USERLIST ')) {
      // user list
      parseUserlist(d.substring(11)); // after '!*USERLIST '
    } else if (d.startsWith('!*MSG ')) {
      popup_message(d.substring(6)); // after '!*MSG '
    } else {
      log(d);
    }
  };
  ws.onerror = function (ev) {
    console.log(ev);
    if (!ws_connected) {
      popup_message("Failed to connect to chat server");
      log('error: failed to connect to server');
    } else {
      popup_message("Error occured: " + ev.data);
      log('error: ' + ev.data);
    }
    show('#connect');
    hide('#interact');
  };
}
document.querySelector('#form')
  .addEventListener('submit', function (evt) {
    msg = document.querySelector("#message");
    ws.send(msg.value);
    msg.value = '';
    evt.preventDefault();
  });
