<?php
/*
 *Add a shortcode [chat-plugin] for inserting the HTML for the plugin
 */


add_shortcode("chat-plugin", "create_chat_plugin");

function create_chat_plugin() {
	chat_styles_and_scripts_enqueue();
	$user = wp_get_current_user();
	$ret["ts"] = time();
	if ($user->exists()) {
		$ret["login"] = $user->user_login;
		$ret["channel"] = "/us/mo/stl";
		$caps = $user->get_role_caps();
		if (isset($caps["administrator"]) && $caps["administrator"]) {
			$ret["admin"] = true;
		}
		$ret["display"] = $user->display_name;
		$ret["first_last_name"] = "$user->first_name $user->last_name";
		$ret["username"] = $user->user_login;
	} else {
		$ret["err"] = 'user not logged in';
	}
	$out = json_encode($ret);
	$hash = hash_hmac("sha256",$out, "mysecretkey");	
  ?>
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
		<script type="javascript">
			letsgo('<?php echo addslashes($out);?>',"<?php echo $hash;?>");</script>
	
  <?php
}

function chat_styles_and_scripts_reg() {
	wp_register_style( 'chat-plugin-css', plugins_url('/assets/chat.css', dirname(__FILE__)));	
	wp_register_script( 'chat-plugin-js',plugins_url ('/assets/chat.js', dirname(__FILE__)));
	
}
add_action('wp_enqueue_scripts', 'chat_styles_and_scripts_reg');

function chat_styles_and_scripts_enqueue() {
	wp_enqueue_style ('chat-plugin-css');
	wp_enqueue_script('chat-plugin-js');
}
?>