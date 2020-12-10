<?php
// https://wordpress.stackexchange.com/questions/76465/initialize-wordpress-environment-to-use-in-command-line-script

define( 'BASE_PATH', __DIR__ . "/" );
define('WP_USE_THEMES', false);
global $wp, $wp_query, $wp_the_query, $wp_rewrite, $wp_did_header;

require_once(BASE_PATH . '/wp-load.php');
$user = wp_get_current_user();
$ret["ts"] = time();
if ($user->exists()) {
	$ret["login"] = $user->user_login;
	$ret["email"] = $user->user_email;
	$ret["display"] = $user->display_name;
} else {
	$ret["err"] = 'user not logged in';
}
$out = json_encode($ret);
$hash = hash_hmac("sha256",$out, "mysecretkey");
echo $out."\n".$hash;
?>
