<?php
/*
 *Add a shortcode [chat-plugin] for inserting the HTML for the plugin
 */


add_shortcode("chat-plugin", "create_chat_plugin");

function create_chat_plugin() {
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
		<script type="javascript">letsgo();</script>
	
  <?php
}


?>