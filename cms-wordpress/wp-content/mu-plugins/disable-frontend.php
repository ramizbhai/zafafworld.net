<?php
/**
 * Plugin Name: Disable WordPress Frontend
 * Description: Redirects all public-facing WordPress page requests to the
 *              SvelteKit discover page. This is the PHP-level backstop for
 *              the Nginx-level redirect in cms-internal.conf.
 *
 *              Belt-and-suspenders design: if Nginx config drifts, this plugin
 *              ensures no WordPress theme/archive/search page is ever served
 *              publicly — the site's front-end is owned entirely by SvelteKit.
 *
 *              Whitelisted: /wp-admin, /wp-login.php, wp-json REST API,
 *              and static file requests (wp-content, wp-includes).
 *
 * Version:     1.0.0
 * Author:      ZafafWorld Engineering
 *
 * Must-use plugin — place in wp-content/mu-plugins/. No activation required.
 */

if ( ! defined( 'ABSPATH' ) ) {
    exit;
}

add_action( 'template_redirect', function (): void {
    // Allow REST API responses through (WP JSON API used by SvelteKit).
    if ( defined( 'REST_REQUEST' ) && REST_REQUEST ) {
        return;
    }

    // Fallback: Allow REST API requests even if REST_REQUEST constant is not yet set
    if ( str_contains( $_SERVER['REQUEST_URI'] ?? '', '/wp-json/' ) ) {
        return;
    }

    // Allow the WP Admin and login page through.
    if ( is_admin() ) {
        return;
    }

    // Allow the login screen through (is_admin() doesn't catch /wp-login.php from the front).
    $request_uri = $_SERVER['REQUEST_URI'] ?? '';
    if ( str_contains( $request_uri, '/wp-login.php' ) ) {
        return;
    }

    // Redirect all remaining public front-end requests to the SvelteKit discover page.
    // 302 (temporary) rather than 301 (permanent) — if this policy ever changes,
    // we don't want browsers caching stale redirects.
    wp_redirect( 'https://zafafworld.net/discover', 302 );
    exit;
} );
