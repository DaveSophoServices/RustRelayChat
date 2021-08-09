<?php define( 'BASE_PATH', __DIR__ . "/" );
define('WP_USE_THEMES', false);
global $wp, $wp_query, $wp_the_query, $wp_rewrite, $wp_did_header;

require_once(BASE_PATH . '/wp-load.php');
get_header(); ?>
	<script>
		var c = { "dev": 
					{ "WP_SERVER":"https://wp.sophoservices.com/get_username.php",
					  "CHAT_SERVER":"ws://localhost:9001"
					},
				   "stage":
					{ "WP_SERVER":"https://wp.sophoservices.com/get_username.php",
					  "CHAT_SERVER":"ws://chat.sophoservices.com:9001"
					}
				}
		function update_config() {
			var mode = getEl("#mode_select").value;
			WP_SERVER=c[mode]['WP_SERVER'];
			CHAT_SERVER=c[mode]['CHAT_SERVER'];
		}				
 		var WP_SERVER=c['dev']['WP_SERVER'];
		var CHAT_SERVER=c['dev']['CHAT_SERVER'];
	</script>
	<style>
		#msg-div, #userlist-div {
			display:none;
			position:absolute;
			border:1px solid black;
			background-color: white;
			max-height:80%;
			padding:30px;
			overflow:auto;
			border-radius:30px;
		}
		#msg-div {
			margin:auto;
			width:auto;
			left:50%;
			top:50%;
			/* use above left,top reference to center of div */
			transform: translate(-50%,-50%); 
		}
	</style>

	<div class="container">
		<select id='mode_select' onchange="update_config()">
			<option value="dev">dev</option>
			<option value="stage">stage</option>
		</select>
		<section id="connect" class="row" style="margin-bottom:10px">
			<button onClick="letsgo()">Connect</button>
		</section>
		<div id="userlist-div">
			<div style='text-align:right;cursor:pointer' onclick='hide("#userlist-div")'>X</div>
			<div id="userlist"></div>
		</div>
		<div id="msg-div">
			<div style="text-align:right;cursor:pointer" onclick='hide("#msg-div")'>X</div>
			<div style="text-align:center;font-size:large" id="message-text"></div>
		</div>
		<div id="interact" style="display:none">
			<section class="row" >
				<form id="form">
					<input type="text" name="message" id="message">
					<input type="submit" value="Send">
				</form>
			</section>
			<section class="row">
				<div><span id="users" >...</span> users online. <a href="#" onclick="getUserList()">[Expand]</a></div>
			</section>
		</div>
		<section class="row border bg-light px-2">
			<pre id="log"></pre>
		</section>
	</div>
	<script type="text/javascript">
		function log(msg) {
			var log=document.querySelector('#log');
			log.innerText = log.innerText + msg + "\n";
		}
		function getEl(f) {
			return document.querySelector(f);
		}
		function getField(f) {
			return document.querySelector(f).value;
		}
		function updateField(f, t) {
			document.querySelector(f).innerText=t;
		}
		function updateFieldHTML(f,t) {
			document.querySelector(f).innerHTML=t;
		}
		function show(f) {
			document.querySelector(f).style.display="block";
		}
		function hide(f) {
			document.querySelector(f).style.display="none";
		}
		function parseStat(stats) {
			try {
				x = JSON.parse(stats);
				updateField("#users", x.users);
			} catch(err) {
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
			fetch(WP_SERVER, { credentials: "include", cache: "no-store"})
			.then(function(r) {
				if (!r.ok) { throw new Error(`HTTP error! status: ${r.status}`); }
				return r.text();
			})
			.then(function(t) {
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
			msg.value='';
			evt.preventDefault();
		} );
		
	</script>
<?php get_sidebar();
get_footer();
?>