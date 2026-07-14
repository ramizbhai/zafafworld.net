<?php
/**
 * Plugin Name: Editor Dashboard & Menu Cleanup
 * Description: Cleans up the WordPress dashboard and hides unnecessary system menus for content editors.
 * Version:     1.0.0
 * Author:      ZafafWorld Engineering
 * Must-use plugin — place in wp-content/mu-plugins/.
 */

if ( ! defined( 'ABSPATH' ) ) {
    exit;
}

/**
 * Remove unnecessary dashboard widgets for non-administrator content editors and Rasha.
 */
add_action( 'wp_dashboard_setup', 'zafaf_cleanup_editor_dashboard_widgets', 999 );
function zafaf_cleanup_editor_dashboard_widgets(): void {
    $current_user = wp_get_current_user();
    // Apply cleanup to non-administrators OR explicitly to Rasha's account
    if ( ! current_user_can( 'manage_options' ) || ( $current_user && 'rasha_marketing' === $current_user->user_login ) ) {
        // Remove Quick Draft
        remove_meta_box( 'dashboard_quick_press', 'dashboard', 'side' );
        
        // Remove WordPress Events and News
        remove_meta_box( 'dashboard_primary', 'dashboard', 'side' );
        remove_meta_box( 'dashboard_secondary', 'dashboard', 'side' );
        
        // Remove Site Health Status
        remove_meta_box( 'dashboard_site_health', 'dashboard', 'normal' );
    }
}

/**
 * Hide system menus not needed by content managers.
 */
add_action( 'admin_menu', 'zafaf_cleanup_editor_admin_menus', 999 );
function zafaf_cleanup_editor_admin_menus(): void {
    $current_user = wp_get_current_user();
    // Hide menus for non-administrators OR explicitly for Rasha's account
    if ( ! current_user_can( 'manage_options' ) || ( $current_user && 'rasha_marketing' === $current_user->user_login ) ) {
        // Hide Tools and Settings menu for safety
        remove_menu_page( 'tools.php' );
        remove_menu_page( 'options-general.php' );
    }
}

