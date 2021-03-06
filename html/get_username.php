<?php
// https://wordpress.stackexchange.com/questions/76465/initialize-wordpress-environment-to-use-in-command-line-script

header("Access-Control-Allow-Origin: http://chat.sophoservices.com");
header("Access-Control-Allow-Credentials: true");

define( 'BASE_PATH', __DIR__ . "/" );
define('WP_USE_THEMES', false);
global $wp, $wp_query, $wp_the_query, $wp_rewrite, $wp_did_header;

require_once(BASE_PATH . '/wp-load.php');
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
echo $out."\n".$hash;
?>
