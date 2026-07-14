<?php
/**
 * Plugin Name: ZafafWorld Plugin Cleanup
 * Description: Automatically deactivates and removes unused plugins (Hello Dolly, Akismet, Google Site Kit).
 * Version:     1.0.0
 * Author:      ZafafWorld Engineering
 * Must-use plugin — place in wp-content/mu-plugins/.
 */

if ( ! defined( 'ABSPATH' ) ) {
    exit;
}

add_action( 'admin_init', 'zafaf_cleanup_unused_plugins' );
function zafaf_cleanup_unused_plugins(): void {
    // Check if we already ran this
    if ( get_option( 'zafaf_plugins_cleaned' ) ) {
        return;
    }

    require_once ABSPATH . 'wp-admin/includes/plugin.php';
    require_once ABSPATH . 'wp-admin/includes/file.php';

    $plugins_to_deactivate = array(
        'hello.php',
        'akismet/akismet.php',
        'google-site-kit/google-site-kit.php'
    );

    foreach ( $plugins_to_deactivate as $plugin ) {
        if ( is_plugin_active( $plugin ) ) {
            deactivate_plugins( $plugin );
        }
    }

    // Delete the plugin files/folders
    $plugins_dir = WP_PLUGIN_DIR . '/';
    
    // 1. Hello Dolly
    $hello_file = $plugins_dir . 'hello.php';
    if ( file_exists( $hello_file ) ) {
        @unlink( $hello_file );
    }

    // 2. Akismet
    $akismet_dir = $plugins_dir . 'akismet';
    if ( file_exists( $akismet_dir ) ) {
        zafaf_recursive_delete_dir( $akismet_dir );
    }

    // 3. Google Site Kit
    $site_kit_dir = $plugins_dir . 'google-site-kit';
    if ( file_exists( $site_kit_dir ) ) {
        zafaf_recursive_delete_dir( $site_kit_dir );
    }

    update_option( 'zafaf_plugins_cleaned', 1 );
}

function zafaf_recursive_delete_dir( string $dir ): void {
    if ( ! is_dir( $dir ) ) {
        return;
    }
    $files = array_diff( scandir( $dir ), array( '.', '..' ) );
    foreach ( $files as $file ) {
        $path = $dir . '/' . $file;
        if ( is_dir( $path ) ) {
            zafaf_recursive_delete_dir( $path );
        } else {
            @unlink( $path );
        }
    }
    @rmdir( $dir );
}
