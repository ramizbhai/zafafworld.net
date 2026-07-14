<?php
/**
 * Plugin Name: Zafaf Translation Automation
 * Description: Registers Gutenberg sidebar translation options and REST API translation endpoints.
 * Version:     3.2.0
 * Author:      ZafafWorld Engineering
 * Must-use plugin — place in wp-content/mu-plugins/.
 */

if ( ! defined( 'ABSPATH' ) ) {
    exit;
}

/**
 * Enqueue block editor assets for the translation sidebar.
 */
add_action( 'enqueue_block_editor_assets', 'zafaf_enqueue_translation_assets' );
function zafaf_enqueue_translation_assets(): void {
    wp_enqueue_script(
        'zafaf-translation-sidebar-js',
        content_url( '/mu-plugins/zafaf-translation.js' ),
        array( 'wp-plugins', 'wp-edit-post', 'wp-element', 'wp-components', 'wp-data', 'wp-api-fetch' ),
        '3.2.0',
        true
    );

    // Localize the current post's language, ID, and translation group mapping
    $post_id = isset( $_GET['post'] ) ? intval( $_GET['post'] ) : 0;
    $lang = 'ar'; // Default language is Arabic
    $translations = array();
    if ( $post_id && function_exists( 'pll_get_post_language' ) ) {
        $lang = pll_get_post_language( $post_id, 'slug' ) ?: 'ar';
        if ( function_exists( 'pll_get_post_translations' ) ) {
            $translations = pll_get_post_translations( $post_id );
        }
    }

    wp_localize_script( 'zafaf-translation-sidebar-js', 'zafafTranslationSettings', array(
        'current_lang' => $lang,
        'translations' => $translations,
        'post_id'      => $post_id,
    ) );
}

/**
 * Auto-initialize permalink structure to /%postname%/ (Post name) for pretty REST URLs.
 */
add_action( 'init', 'zafaf_initialize_permalinks', 10 );
function zafaf_initialize_permalinks(): void {
    if ( get_option( 'permalink_structure' ) !== '/%postname%/' ) {
        update_option( 'permalink_structure', '/%postname%/' );
        flush_rewrite_rules();
    }
}

/**
 * Auto-initialize Polylang languages (English & Arabic) if they are not configured.
 * Default language is configured to Arabic (ar).
 */
add_action( 'init', 'zafaf_initialize_polylang_languages', 15 );
function zafaf_initialize_polylang_languages(): void {
    global $polylang;
    if ( ! $polylang || ! isset( $polylang->model ) ) {
        return;
    }

    $languages = $polylang->model->get_languages_list();
    if ( empty( $languages ) ) {
        // Create English Language
        $polylang->model->add_language( array(
            'name'       => 'English',
            'locale'     => 'en_US',
            'slug'       => 'en',
            'rtl'        => 0,
            'term_group' => 0,
        ) );

        // Create Arabic Language (Primary)
        $polylang->model->add_language( array(
            'name'       => 'العربية',
            'locale'     => 'ar',
            'slug'       => 'ar',
            'rtl'        => 1,
            'term_group' => 0,
        ) );
    }

    // Force default language to Arabic (ar) and enable standard post types/taxonomies
    $options = get_option( 'polylang' );
    if ( is_array( $options ) ) {
        $needs_update = false;
        if ( $options['default_lang'] !== 'ar' ) {
            $options['default_lang'] = 'ar';
            $needs_update = true;
        }
        if ( empty( $options['post_types'] ) ) {
            $options['post_types'] = array( 'post' => 'post', 'page' => 'page' );
            $needs_update = true;
        }
        if ( empty( $options['taxonomies'] ) ) {
            $options['taxonomies'] = array( 'category' => 'category', 'post_tag' => 'post_tag' );
            $needs_update = true;
        }

        if ( $needs_update ) {
            update_option( 'polylang', $options );
            $polylang->model->clean_languages_cache();
        }
    }
}

/**
 * Register the REST API route for translation.
 */
add_action( 'rest_api_init', 'zafaf_register_translation_routes' );
function zafaf_register_translation_routes(): void {
    register_rest_route( 'zafaf/v1', '/translate', array(
        'methods'             => 'POST',
        'callback'            => 'zafaf_handle_translation_request',
        'permission_callback' => 'zafaf_translation_permissions_check',
    ) );
    register_rest_route( 'zafaf/v1', '/status', array(
        'methods'             => 'GET',
        'callback'            => 'zafaf_handle_status_request',
        'permission_callback' => 'zafaf_translation_permissions_check',
    ) );
}

/**
 * Validate that the user is an authenticated editor or admin who can edit posts.
 */
function zafaf_translation_permissions_check( WP_REST_Request $request ): bool {
    return current_user_can( 'edit_posts' );
}

/**
 * REST Callback for retrieving the current post language and translation mapping.
 */
function zafaf_handle_status_request( WP_REST_Request $request ): WP_REST_Response {
    $post_id = intval( $request->get_param( 'post_id' ) );
    if ( ! $post_id ) {
        return new WP_REST_Response( array(
            'success' => false,
            'message' => 'Invalid Post ID.',
        ), 400 );
    }

    $lang = 'ar';
    $translations = array();
    if ( function_exists( 'pll_get_post_language' ) ) {
        $lang = pll_get_post_language( $post_id, 'slug' ) ?: 'ar';
        if ( function_exists( 'pll_get_post_translations' ) ) {
            $translations = pll_get_post_translations( $post_id );
        }
    }

    return new WP_REST_Response( array(
        'success'      => true,
        'current_lang' => $lang,
        'translations' => (object) $translations,
        'post_id'      => $post_id,
    ), 200 );
}

/**
 * Interface for translation providers.
 */
interface Zafaf_Translation_Provider {
    public static function translate( string $text, string $target_lang ): string;
}

/**
 * Default translation provider implementation supporting LibreTranslate, Google Translate, and DeepL.
 */
class Zafaf_Default_Translation_Provider implements Zafaf_Translation_Provider {
    public static function translate( string $text, string $target_lang ): string {
        if ( empty( $text ) ) {
            return '';
        }

        // 1. Try LibreTranslate API if configured
        $libre_url = getenv( 'LIBRETRANSLATE_API_URL' ) ?: get_option( 'zafaf_libretranslate_url' );
        $libre_key = getenv( 'LIBRETRANSLATE_API_KEY' ) ?: get_option( 'zafaf_libretranslate_key' );
        if ( ! empty( $libre_url ) ) {
            $source_lang = ( 'en' === $target_lang ) ? 'ar' : 'en';
            $body = array(
                'q'      => $text,
                'source' => $source_lang,
                'target' => $target_lang,
                'format' => 'html',
            );
            if ( ! empty( $libre_key ) ) {
                $body['api_key'] = $libre_key;
            }
            $response = wp_remote_post( rtrim( $libre_url, '/' ) . '/translate', array(
                'headers' => array( 'Content-Type' => 'application/json' ),
                'body'    => wp_json_encode( $body ),
                'timeout' => 10,
            ) );
            if ( ! is_wp_error( $response ) && wp_remote_retrieve_response_code( $response ) === 200 ) {
                $data = json_decode( wp_remote_retrieve_body( $response ), true );
                if ( ! empty( $data['translatedText'] ) ) {
                    return $data['translatedText'];
                }
            }
        }

        // 2. Try Google Translate Free Endpoint Fallback
        $source_lang = ( 'en' === $target_lang ) ? 'ar' : 'en';
        $url = 'https://translate.googleapis.com/translate_a/single?client=gtx&sl=' . $source_lang . '&tl=' . $target_lang . '&dt=t&q=' . rawurlencode( $text );
        $response = wp_remote_get( $url, array( 'timeout' => 10 ) );
        if ( ! is_wp_error( $response ) && wp_remote_retrieve_response_code( $response ) === 200 ) {
            $body = json_decode( wp_remote_retrieve_body( $response ), true );
            if ( isset( $body[0] ) && is_array( $body[0] ) ) {
                $translated = '';
                foreach ( $body[0] as $sentence ) {
                    if ( isset( $sentence[0] ) ) {
                        $translated .= $sentence[0];
                    }
                }
                if ( ! empty( $translated ) ) {
                    return $translated;
                }
            }
        }

        // 3. Try DeepL API if key is present
        $deepl_key = getenv( 'DEEPL_API_KEY' );
        if ( ! empty( $deepl_key ) ) {
            $is_free = str_ends_with( $deepl_key, ':fx' );
            $endpoint = $is_free ? 'https://api-free.deepl.com/v2/translate' : 'https://api.deepl.com/v2/translate';
            $response = wp_remote_post( $endpoint, array(
                'headers' => array(
                    'Authorization' => 'DeepL-Auth-Key ' . $deepl_key,
                    'Content-Type'  => 'application/json',
                ),
                'body' => wp_json_encode( array(
                    'text'        => array( $text ),
                    'target_lang' => strtoupper( $target_lang ),
                ) ),
                'timeout' => 10,
            ) );
            if ( ! is_wp_error( $response ) && wp_remote_retrieve_response_code( $response ) === 200 ) {
                $data = json_decode( wp_remote_retrieve_body( $response ), true );
                if ( isset( $data['translations'][0]['text'] ) ) {
                    return $data['translations'][0]['text'];
                }
            }
        }

        // Final fallback: Prepend target language prefix
        if ( 'ar' === $target_lang ) {
            return 'العربية - ' . $text;
        }
        return 'English - ' . $text;
    }
}

/**
 * Reusable Translation Engine (Bidirectional).
 * Creates or updates the opposing translation post, translates core fields and Rank Math SEO,
 * and copies categories, tags, and images.
 */
function zafaf_perform_translation( int $post_id, string $source_lang, string $target_lang, string $status = 'draft' ): int {
    $post = get_post( $post_id );
    if ( ! $post ) {
        return 0;
    }

    // Translate Core Fields using the provider interface
    $translated_title = zafaf_translate_text( $post->post_title, $target_lang );
    $translated_content = zafaf_translate_content( $post->post_content, $target_lang );
    $translated_excerpt = zafaf_translate_text( $post->post_excerpt, $target_lang );

    // Check if target language translation already exists
    $target_post_id = 0;
    if ( function_exists( 'pll_get_post' ) ) {
        $target_post_id = pll_get_post( $post_id, $target_lang );
    }

    if ( $target_post_id ) {
        // Update existing translation post
        $updated_post_data = array(
            'ID'           => $target_post_id,
            'post_title'   => $translated_title,
            'post_content' => $translated_content,
            'post_excerpt' => $translated_excerpt,
        );
        // Force status to publish if requested
        if ( 'publish' === $status ) {
            $updated_post_data['post_status'] = 'publish';
        }
        wp_update_post( $updated_post_data );
    } else {
        // Create new translation post
        $new_post_data = array(
            'post_title'    => $translated_title,
            'post_name'     => sanitize_title( $translated_title ),
            'post_content'  => $translated_content,
            'post_excerpt'  => $translated_excerpt,
            'post_status'   => $status,
            'post_author'   => $post->post_author,
            'post_type'     => $post->post_type,
        );
        $target_post_id = wp_insert_post( $new_post_data );

        if ( is_wp_error( $target_post_id ) ) {
            return 0;
        }

        // Link the posts as translations in Polylang
        if ( function_exists( 'pll_set_post_language' ) && function_exists( 'pll_save_post_translations' ) ) {
            pll_set_post_language( $post_id, $source_lang );
            pll_set_post_language( $target_post_id, $target_lang );
            
            $translations = array(
                $source_lang => $post_id,
                $target_lang => $target_post_id,
            );
            $existing_translations = pll_get_post_translations( $post_id );
            if ( is_array( $existing_translations ) ) {
                $translations = array_merge( $existing_translations, $translations );
            }
            pll_save_post_translations( $translations );
        }
    }

    // Sync featured image (thumbnail ID)
    $thumbnail_id = get_post_thumbnail_id( $post_id );
    if ( $thumbnail_id ) {
        set_post_thumbnail( $target_post_id, $thumbnail_id );
    } else {
        delete_post_thumbnail( $target_post_id );
    }

    // Sync categories mapping (with Polylang translations if available)
    $categories = wp_get_post_categories( $post_id );
    $target_categories = array();
    foreach ( $categories as $cat_id ) {
        if ( function_exists( 'pll_get_term' ) ) {
            $translated_cat_id = pll_get_term( $cat_id, $target_lang );
            $target_categories[] = $translated_cat_id ?: $cat_id;
        } else {
            $target_categories[] = $cat_id;
        }
    }
    wp_set_post_categories( $target_post_id, $target_categories );

    // Sync tags mapping (with Polylang translations if available)
    $tags = wp_get_post_tags( $post_id );
    $target_tags = array();
    foreach ( $tags as $tag_obj ) {
        if ( function_exists( 'pll_get_term' ) ) {
            $translated_tag_id = pll_get_term( $tag_obj->term_id, $target_lang );
            $target_tags[] = $translated_tag_id ?: $tag_obj->term_id;
        } else {
            $target_tags[] = $tag_obj->term_id;
        }
    }
    wp_set_post_tags( $target_post_id, $target_tags );

    // Sync Rank Math SEO fields (copies keywords, description, schema, social images)
    zafaf_sync_rank_math_meta( $post_id, $target_post_id, $target_lang );

    return $target_post_id;
}

/**
 * REST Callback for handling translation request (Arabic <-> English bidirectional).
 */
function zafaf_handle_translation_request( WP_REST_Request $request ): WP_REST_Response {
    $params = $request->get_json_params();
    $post_id = isset( $params['post_id'] ) ? intval( $params['post_id'] ) : 0;

    if ( ! $post_id ) {
        return new WP_REST_Response( array(
            'success' => false,
            'message' => 'Invalid Post ID.',
        ), 400 );
    }

    $post = get_post( $post_id );
    if ( ! $post ) {
        return new WP_REST_Response( array(
            'success' => false,
            'message' => 'Post not found.',
        ), 404 );
    }

    // Determine translation direction based on Polylang
    $source_lang = 'ar';
    $target_lang = 'en';
    if ( function_exists( 'pll_get_post_language' ) ) {
        $source_lang = pll_get_post_language( $post_id, 'slug' ) ?: 'ar';
        if ( 'en' === $source_lang ) {
            $target_lang = 'ar';
        }
    }

    // Perform translation matching the source post status
    $target_post_id = zafaf_perform_translation( $post_id, $source_lang, $target_lang, $post->post_status );

    if ( ! $target_post_id ) {
        return new WP_REST_Response( array(
            'success' => false,
            'message' => 'Failed to create or update translation post.',
        ), 500 );
    }

    // Build the admin redirect URL to review the target translation
    $redirect_url = admin_url( 'post.php?post=' . $target_post_id . '&action=edit' );

    return new WP_REST_Response( array(
        'success'        => true,
        'target_post_id' => $target_post_id,
        'redirect_url'   => $redirect_url,
    ), 200 );
}

/**
 * Phase 3: Automatic Translation Safety System.
 * Hooked at priority 5 on transition_post_status so it fires before the webhook (priority 10).
 */
add_action( 'transition_post_status', 'zafaf_auto_translate_on_publish', 5, 3 );
function zafaf_auto_translate_on_publish( string $new_status, string $old_status, WP_Post $post ): void {
    if ( 'post' !== $post->post_type ) {
        return;
    }

    if ( 'publish' !== $new_status ) {
        return;
    }

    // Prevent infinite recursion loops
    static $running = false;
    if ( $running ) {
        return;
    }
    $running = true;

    $post_id = $post->ID;

    // Determine language of current post
    $source_lang = 'ar';
    $target_lang = 'en';
    if ( function_exists( 'pll_get_post_language' ) ) {
        $source_lang = pll_get_post_language( $post_id, 'slug' ) ?: 'ar';
        if ( 'en' === $source_lang ) {
            $target_lang = 'ar';
        }
    }

    // Check if target translation exists
    $target_post_id = 0;
    if ( function_exists( 'pll_get_post' ) ) {
        $target_post_id = pll_get_post( $post_id, $target_lang );
    }

    if ( ! $target_post_id ) {
        // Translation is missing! Automatically generate and publish it.
        zafaf_perform_translation( $post_id, $source_lang, $target_lang, 'publish' );
    } else {
        // Translation exists, ensure it is also published
        $target_post = get_post( $target_post_id );
        if ( $target_post && 'publish' !== $target_post->post_status ) {
            wp_update_post( array(
                'ID'          => $target_post_id,
                'post_status' => 'publish',
            ) );
        }
    }

    $running = false;
}

/**
 * Wrapper for translation calling Zafaf_Default_Translation_Provider.
 */
function zafaf_translate_text( $text, $target_lang = 'en' ): string {
    return Zafaf_Default_Translation_Provider::translate( $text, $target_lang );
}

/**
 * Smart translator for HTML content block structures.
 * Preserves Gutenberg HTML comments and structural tags while translating internal text.
 */
function zafaf_translate_content( $content, $target_lang = 'en' ): string {
    $libre_url = getenv( 'LIBRETRANSLATE_API_URL' );
    $deepl_key = getenv( 'DEEPL_API_KEY' );
    $google_key = getenv( 'GOOGLE_TRANSLATION_API_KEY' );
    
    if ( ! empty( $libre_url ) || ! empty( $deepl_key ) || ! empty( $google_key ) ) {
        return Zafaf_Default_Translation_Provider::translate( $content, $target_lang );
    }

    // Fallback: Translate text nodes inside HTML tags
    $callback = function( $matches ) use ( $target_lang ) {
        $tag = $matches[1];
        $text = trim( $matches[2] );
        if ( empty( $text ) || is_numeric( $text ) ) {
            return $matches[0];
        }
        $translated_text = Zafaf_Default_Translation_Provider::translate( $text, $target_lang );
        return $tag . $translated_text;
    };
    return preg_replace_callback( '/(<[^>]+>)([^<]+)/', $callback, $content );
}

/**
 * Copy and translate Rank Math SEO fields between translation posts (Phase 6).
 * Syncs Focus Keyword, SEO Title, Description, robots, schema snippet, and Social Media (OpenGraph) images.
 */
function zafaf_sync_rank_math_meta( int $source_id, int $target_id, string $target_lang ): void {
    global $wpdb;
    
    // Find all meta rows starting with rank_math_ for the source post
    $meta_rows = $wpdb->get_results( $wpdb->prepare(
        "SELECT meta_key, meta_value FROM {$wpdb->postmeta} WHERE post_id = %d AND meta_key LIKE 'rank_math_%%'",
        $source_id
    ) );

    foreach ( $meta_rows as $row ) {
        $key = $row->meta_key;
        $val = $row->meta_value;

        // Skip post-specific analytics keys
        if ( in_array( $key, array( 'rank_math_analytic_object_id' ), true ) ) {
            continue;
        }

        // Translate text-based SEO fields
        if ( 'rank_math_title' === $key ) {
            $val = zafaf_translate_text( $val, $target_lang );
        } elseif ( 'rank_math_description' === $key ) {
            $val = zafaf_translate_text( $val, $target_lang );
        } elseif ( 'rank_math_focus_keyword' === $key ) {
            $val = zafaf_translate_text( $val, $target_lang );
        }

        // Update the metadata on the target translation post
        update_post_meta( $target_id, $key, $val );
    }
}

/**
 * Automatically delete linked translations when a post is deleted.
 */
add_action( 'before_delete_post', 'zafaf_delete_translations_on_post_delete', 10, 1 );
function zafaf_delete_translations_on_post_delete( int $post_id ): void {
    static $deleting = false;
    if ( $deleting ) {
        return;
    }

    if ( ! function_exists( 'pll_get_post_translations' ) ) {
        return;
    }

    $translations = pll_get_post_translations( $post_id );
    if ( empty( $translations ) ) {
        return;
    }

    $deleting = true;
    foreach ( $translations as $lang => $translated_id ) {
        if ( $translated_id !== $post_id ) {
            wp_delete_post( $translated_id, true );
        }
    }
    $deleting = false;
}

/**
 * Automatically trash linked translations when a post is trashed.
 */
add_action( 'wp_trash_post', 'zafaf_trash_translations_on_post_trash', 10, 1 );
function zafaf_trash_translations_on_post_trash( int $post_id ): void {
    static $trashing = false;
    if ( $trashing ) {
        return;
    }

    if ( ! function_exists( 'pll_get_post_translations' ) ) {
        return;
    }

    $translations = pll_get_post_translations( $post_id );
    if ( empty( $translations ) ) {
        return;
    }

    $trashing = true;
    foreach ( $translations as $lang => $translated_id ) {
        if ( $translated_id !== $post_id ) {
            wp_trash_post( $translated_id );
        }
    }
    $trashing = false;
}

/**
 * Automatically restore linked translations when a post is untrashed.
 */
add_action( 'untrash_post', 'zafaf_untrash_translations_on_post_untrash', 10, 1 );
function zafaf_untrash_translations_on_post_untrash( int $post_id ): void {
    static $untrashing = false;
    if ( $untrashing ) {
        return;
    }

    if ( ! function_exists( 'pll_get_post_translations' ) ) {
        return;
    }

    $translations = pll_get_post_translations( $post_id );
    if ( empty( $translations ) ) {
        return;
    }

    $untrashing = true;
    foreach ( $translations as $lang => $translated_id ) {
        if ( $translated_id !== $post_id ) {
            wp_untrash_post( $translated_id );
        }
    }
    $untrashing = false;
}
