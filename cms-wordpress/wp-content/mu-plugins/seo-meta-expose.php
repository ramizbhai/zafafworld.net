<?php
/**
 * Plugin Name: SEO Meta Expose
 * Description: Exposes Rank Math SEO metadata as `seo_meta_payload` on the WP REST API.
 *              Consumed by the SvelteKit server loader (discover/[slug]/+page.server.ts)
 *              for Open Graph tags, canonical URLs, and JSON-LD structured data.
 * Version:     1.0.0
 * Author:      ZafafWorld Engineering
 *
 * Must-use plugin — place in wp-content/mu-plugins/. No activation required.
 *
 * Field name: `seo_meta_payload`
 * This name matches the SvelteKit loader's expectation (wpPost.seo_meta_payload).
 * Do NOT rename without updating the loader simultaneously.
 */

if ( ! defined( 'ABSPATH' ) ) {
    exit; // Do not allow direct file execution.
}

add_action( 'rest_api_init', function () {
    register_rest_field( 'post', 'seo_meta_payload', [
        'get_callback' => function ( $post_arr ) {
            $id = $post_arr['id'];

            // ── JSON-LD structured data (Rank Math) ─────────────────────────
            $json_ld = [];
            if ( function_exists( 'rank_math' ) ) {
                try {
                    // Rank Math exposes structured data through its JSON-LD service.
                    $json_ld_instance = rank_math()->json_ld ?? null;
                    if ( $json_ld_instance && method_exists( $json_ld_instance, 'get_data' ) ) {
                        $json_ld = $json_ld_instance->get_data();
                    }
                } catch ( \Throwable $e ) {
                    // Rank Math JSON-LD retrieval is best-effort — never fail the REST response.
                    $json_ld = [];
                }
            }

            // ── OG image ────────────────────────────────────────────────────
            $og_image = get_post_meta( $id, 'rank_math_facebook_image', true );
            if ( empty( $og_image ) ) {
                // Fall back to the post's featured image URL.
                $thumbnail_id = get_post_thumbnail_id( $id );
                if ( $thumbnail_id ) {
                    $og_image = wp_get_attachment_image_url( $thumbnail_id, 'full' ) ?: '';
                }
            }

            return [
                'title'         => get_post_meta( $id, 'rank_math_title', true )
                                    ?: get_the_title( $id ),
                'description'   => get_post_meta( $id, 'rank_math_description', true )
                                    ?: wp_strip_all_tags( get_the_excerpt( $id ) ),
                'robots'        => get_post_meta( $id, 'rank_math_robots', true )
                                    ?: [ 'index', 'follow' ],
                'canonical_url' => get_post_meta( $id, 'rank_math_canonical_url', true )
                                    ?: get_permalink( $id ),
                'og_image'      => $og_image ?: '',
                'focus_keyword' => get_post_meta( $id, 'rank_math_focus_keyword', true ) ?: '',
                'json_ld'       => $json_ld,
            ];
        },
        'update_callback' => null, // Read-only field — no write support via REST.
        'schema'          => [
            'type'        => 'object',
            'description' => 'SEO metadata sourced from Rank Math for the SvelteKit consumer.',
        ],
    ] );
} );
