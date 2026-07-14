<?php
/**
 * Plugin Name: Sync Webhook — Rust Backend Bridge
 * Description: Fires a fire-and-forget webhook to the Rust backend's internal
 *              sync endpoint whenever a post is published or updated to published.
 *              This keeps the Postgres `blogs` shadow table (source='wordpress')
 *              in sync so that blog_views, blog_comments, and blog_funnel_events
 *              FKs remain intact without touching those tables.
 * Version:     1.0.0
 * Author:      ZafafWorld Engineering
 *
 * Must-use plugin — place in wp-content/mu-plugins/. No activation required.
 *
 * IMPORTANT: The `WP_SYNC_SECRET` PHP constant must be defined in wp-config.php
 *            (injected via WORDPRESS_CONFIG_EXTRA in the Podman Compose env block).
 *            Never hardcode or expose this value.
 *
 * Target endpoint: http://backend:8080/api/v1/internal/wp-sync
 *   - `backend` is the Podman Compose service name for the Rust backend.
 *   - This URL is only resolvable inside zafaf_network. It is NOT a public URL.
 *   - `blocking: false` makes this a fire-and-forget call that does not slow
 *     down the editor's Publish button.
 */

if ( ! defined( 'ABSPATH' ) ) {
    exit;
}

/**
 * Fires on every post status transition. We only act when a `post` type
 * transitions TO `publish` (new publish) or when it's already published and
 * gets updated (transition from publish → publish), or when it transitions FROM
 * `publish` to something else (e.g. trash, draft) where we unpublish/delete it.
 */
add_action( 'transition_post_status', 'zafaf_sync_wp_post_to_rust', 10, 3 );
add_action( 'wp_trash_post', 'zafaf_trash_wp_post_in_rust', 10, 1 );
add_action( 'before_delete_post', 'zafaf_delete_wp_post_in_rust', 10, 1 );

function zafaf_trash_wp_post_in_rust( int $post_id ): void {
    $post = get_post( $post_id );
    if ( ! $post || 'post' !== $post->post_type ) {
        return;
    }
    zafaf_send_action_to_rust( $post_id, 'delete' );
}

function zafaf_delete_wp_post_in_rust( int $post_id ): void {
    $post = get_post( $post_id );
    if ( ! $post || 'post' !== $post->post_type ) {
        return;
    }
    zafaf_send_action_to_rust( $post_id, 'delete' );
}

function zafaf_send_action_to_rust( int $wp_post_id, string $action ): void {
    $payload = wp_json_encode( [
        'wp_post_id' => $wp_post_id,
        'action'     => $action,
    ] );

    $secret = defined( 'WP_SYNC_SECRET' ) ? WP_SYNC_SECRET : '';
    if ( empty( $secret ) ) {
        error_log( '[ZafafWorld] WP_SYNC_SECRET is not defined — delete webhook skipped for post ID ' . $wp_post_id );
        return;
    }

    wp_remote_post(
        'http://backend:8080/api/v1/internal/wp-sync',
        [
            'headers'  => [
                'Content-Type'     => 'application/json',
                'X-Webhook-Secret' => $secret,
            ],
            'body'     => $payload,
            'timeout'  => 5,
            'blocking' => false,
        ]
    );
}

function zafaf_sync_wp_post_to_rust( string $new_status, string $old_status, WP_Post $post ): void {
    // Only sync `post` post-type. Skip pages, attachments, custom types, etc.
    if ( 'post' !== $post->post_type ) {
        return;
    }

    // Only act when the post is being published (new) or re-saved while already published.
    $is_publishing = ( 'publish' === $new_status );
    if ( ! $is_publishing ) {
        // If the post was previously published and is now unpublished (trash, draft, private, etc.), send delete action
        if ( 'publish' === $old_status ) {
            zafaf_send_action_to_rust( $post->ID, 'delete' );
        }
        return;
    }

    // ── Build the bilingual title payload ─────────────────────────────────────
    // Polylang: If Polylang is active, resolve per-language titles.
    // Falls back gracefully to the post_title if Polylang is not installed.
    $title_en = '';
    $title_ar = '';

    if ( function_exists( 'pll_get_post_language' ) ) {
        $lang = pll_get_post_language( $post->ID, 'slug' ) ?: 'ar'; // 'en' or 'ar'

        if ( 'ar' === $lang ) {
            $title_ar = $post->post_title;
            // Attempt to find the English translation
            $en_post_id = pll_get_post( $post->ID, 'en' );
            if ( $en_post_id ) {
                $en_post  = get_post( $en_post_id );
                $title_en = $en_post ? $en_post->post_title : '';
            }
        } else {
            $title_en = $post->post_title;
            // Attempt to find the Arabic translation
            $ar_post_id = pll_get_post( $post->ID, 'ar' );
            if ( $ar_post_id ) {
                $ar_post  = get_post( $ar_post_id );
                $title_ar = $ar_post ? $ar_post->post_title : '';
            }
        }
    } else {
        // No Polylang — use post_title as the English title; Arabic is empty.
        $title_en = $post->post_title;
        $title_ar = '';
    }

    // ── Build the published_at timestamp ──────────────────────────────────────
    // Ensure we do not send invalid '0000-00-00 00:00:00' dates to the database.
    // post_date_gmt is already UTC; format as ISO 8601 for Rust's chrono parser.
    $published_at = '';
    if ( ! empty( $post->post_date_gmt ) && '0000-00-00 00:00:00' !== $post->post_date_gmt ) {
        $published_at = gmdate( 'c', strtotime( $post->post_date_gmt ) );
    } elseif ( ! empty( $post->post_date ) && '0000-00-00 00:00:00' !== $post->post_date ) {
        $published_at = gmdate( 'c', strtotime( $post->post_date ) );
    } else {
        $published_at = gmdate( 'c' );
    }

    // ── Build categories list ────────────────────────────────────────────────
    $categories = [];
    $wp_cats = get_the_category( $post->ID );
    if ( is_array( $wp_cats ) ) {
        foreach ( $wp_cats as $cat ) {
            $categories[] = [
                'name' => $cat->name,
                'slug' => $cat->slug,
            ];
        }
    }

    // ── Build tags list ──────────────────────────────────────────────────────
    $tags = [];
    $wp_tags = get_the_tags( $post->ID );
    if ( is_array( $wp_tags ) ) {
        foreach ( $wp_tags as $tag ) {
            $tags[] = [
                'name' => $tag->name,
                'slug' => $tag->slug,
            ];
        }
    }

    // ── Build Polylang language and translation group ID ─────────────────────
    $lang = 'ar';
    $translation_group_id = $post->ID; // default to self
    if ( function_exists( 'pll_get_post_language' ) ) {
        $lang = pll_get_post_language( $post->ID, 'slug' ) ?: 'ar';
        $translations = pll_get_post_translations( $post->ID );
        if ( is_array( $translations ) && isset( $translations['ar'] ) ) {
            $translation_group_id = (int) $translations['ar'];
        } elseif ( is_array( $translations ) && isset( $translations['en'] ) ) {
            $translation_group_id = (int) $translations['en'];
        } elseif ( is_array( $translations ) && !empty( $translations ) ) {
            $translation_group_id = (int) min( $translations );
        }
    }

    // ── Build Rank Math SEO fields ───────────────────────────────────────────
    $meta_title = get_post_meta( $post->ID, 'rank_math_title', true ) ?: $post->post_title;
    $meta_description = get_post_meta( $post->ID, 'rank_math_description', true ) ?: wp_strip_all_tags( get_the_excerpt( $post ) );
    $focus_keywords = get_post_meta( $post->ID, 'rank_math_focus_keyword', true ) ?: '';
    $canonical_url = get_post_meta( $post->ID, 'rank_math_canonical_url', true ) ?: get_permalink( $post->ID );

    // ── Extract and Parse Content (Gutenberg to JSON) ─────────────────────────
    $local_upload_prefix = 'https://blog.zafafworld.net/wp-content/uploads/';
    $cdn_upload_prefix = 'https://api.zafafworld.net/assets/uploads/';

    $content_html = '';
    if ( has_blocks( $post->post_content ) ) {
        $parsed_blocks = parse_blocks( $post->post_content );
        $json_blocks = [];

        foreach ( $parsed_blocks as $block ) {
            if ( empty( $block['blockName'] ) ) {
                continue;
            }

            $type = '';
            $content = '';
            $url = '';
            $layout = '';

            switch ( $block['blockName'] ) {
                case 'core/heading':
                    $level = $block['attrs']['level'] ?? 2;
                    $type = ( $level <= 2 ) ? 'heading' : 'subheading';
                    $content = wp_strip_all_tags( $block['innerHTML'] );
                    break;
                case 'core/paragraph':
                    $type = 'text';
                    $content = wp_strip_all_tags( $block['innerHTML'] );
                    break;
                case 'core/list':
                    $type = 'list';
                    preg_match_all( '/<li>(.*?)<\/li>/is', $block['innerHTML'], $matches );
                    if ( ! empty( $matches[1] ) ) {
                        $items = array_map( 'wp_strip_all_tags', $matches[1] );
                        $content = implode( "\n", $items );
                    } else {
                        $content = wp_strip_all_tags( $block['innerHTML'] );
                    }
                    break;
                case 'core/image':
                    $type = 'image';
                    preg_match( '/src=["\'](.*?)["\']/', $block['innerHTML'], $matches );
                    if ( ! empty( $matches[1] ) ) {
                        $url = $matches[1];
                    }
                    break;
                case 'core/gallery':
                    $type = 'gallery';
                    preg_match_all( '/src=["\'](.*?)["\']/', $block['innerHTML'], $matches );
                    if ( ! empty( $matches[1] ) ) {
                        $url = implode( "\n", $matches[1] );
                    }
                    break;
                case 'core/separator':
                    $type = 'divider';
                    break;
                case 'core/media-text':
                    $type = 'image_text';
                    $layout = isset( $block['attrs']['mediaPosition'] ) && 'right' === $block['attrs']['mediaPosition'] ? 'right' : 'left';
                    preg_match( '/src=["\'](.*?)["\']/', $block['innerHTML'], $matches );
                    if ( ! empty( $matches[1] ) ) {
                        $url = $matches[1];
                    }
                    if ( ! empty( $block['innerBlocks'] ) ) {
                        $texts = [];
                        foreach ( $block['innerBlocks'] as $inner ) {
                            if ( 'core/paragraph' === $inner['blockName'] || 'core/heading' === $inner['blockName'] ) {
                                $texts[] = wp_strip_all_tags( $inner['innerHTML'] );
                            }
                        }
                        $content = implode( "\n\n", $texts );
                    }
                    break;
            }

            if ( ! empty( $type ) ) {
                $parsed_block = [ 'type' => $type ];
                if ( ! empty( $content ) ) {
                    // Decode entities so they show properly in JSON string
                    $parsed_block['content'] = html_entity_decode( $content, ENT_QUOTES, 'UTF-8' );
                }
                if ( ! empty( $url ) ) {
                    $parsed_block['url'] = str_replace( $local_upload_prefix, $cdn_upload_prefix, $url );
                }
                if ( ! empty( $layout ) ) {
                    $parsed_block['layout'] = $layout;
                }
                $json_blocks[] = $parsed_block;
            }
        }
        $content_html = wp_json_encode( $json_blocks );
    } else {
        // Fallback to raw HTML for Classic Editor
        $content_html = apply_filters( 'the_content', $post->post_content );
        $content_html = str_replace( $local_upload_prefix, $cdn_upload_prefix, $content_html );
    }

    $excerpt = get_the_excerpt( $post );
    $cover_image_url = get_the_post_thumbnail_url( $post->ID, 'full' ) ?: '';
    
    $cover_image_url = str_replace( $local_upload_prefix, $cdn_upload_prefix, $cover_image_url );
    $meta_description = str_replace( $local_upload_prefix, $cdn_upload_prefix, $meta_description );
    $canonical_url = str_replace( $local_upload_prefix, $cdn_upload_prefix, $canonical_url );

    // ── Prepare the payload ───────────────────────────────────────────────────
    $payload = wp_json_encode( [
        'wp_post_id'           => $post->ID,
        'action'               => 'publish',
        'slug'                 => urldecode( $post->post_name ),
        'title_en'             => $title_en,
        'title_ar'             => $title_ar,
        'published_at'         => $published_at,
        'content_html'         => $content_html,
        'excerpt'              => $excerpt,
        'cover_image_url'      => $cover_image_url,
        'categories'           => $categories,
        'tags'                 => $tags,
        'lang'                 => $lang,
        'translation_group_id' => $translation_group_id,
        'meta_title'           => $meta_title,
        'meta_description'     => $meta_description,
        'focus_keywords'       => $focus_keywords,
        'canonical_url'        => $canonical_url,
    ] );

    error_log( '[ZafafWorld] Webhook Payload: ' . $payload );

    // ── Retrieve the shared secret ────────────────────────────────────────────
    // Defined via WORDPRESS_CONFIG_EXTRA in Compose → wp-config.php.
    $secret = defined( 'WP_SYNC_SECRET' ) ? WP_SYNC_SECRET : '';

    if ( empty( $secret ) ) {
        // Log the misconfiguration but don't crash the publish flow.
        error_log( '[ZafafWorld] WP_SYNC_SECRET is not defined — sync webhook skipped for post ID ' . $post->ID );
        return;
    }

    // ── Fire the internal webhook (fire-and-forget) ───────────────────────────
    // `backend` is the Compose service name. This URL ONLY resolves inside zafaf_network.
    // `blocking: false` returns immediately — the editor's Publish button is not held.
    $response = wp_remote_post(
        'http://backend:8080/api/v1/internal/wp-sync',
        [
            'headers'  => [
                'Content-Type'     => 'application/json',
                'X-Webhook-Secret' => $secret,
            ],
            'body'     => $payload,
            'timeout'  => 5,
            'blocking' => false, // Fire-and-forget for reliability
        ]
    );

    // `blocking: false` means $response is always WP_Error with "could not connect" (expected).
    // Log real errors only in debug mode to avoid noise.
    if ( defined( 'WP_DEBUG' ) && WP_DEBUG && is_wp_error( $response ) ) {
        error_log( '[ZafafWorld] Sync webhook dispatch: ' . $response->get_error_message() );
    }
}
