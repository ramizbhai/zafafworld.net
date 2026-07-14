<?php
/**
 * Plugin Name: ZafafWorld WordPress Media Sync
 * Description: Hooks into attachment lifecycle and sends them to the Rust backend to upload to MinIO.
 * Version:     1.0.0
 * Author:      ZafafWorld Engineering
 * Must-use plugin — place in wp-content/mu-plugins/.
 */

if ( ! defined( 'ABSPATH' ) ) {
    exit;
}

// Hook into attachment lifecycle
add_action( 'add_attachment', 'zafaf_sync_media_on_add', 10, 1 );
add_action( 'edit_attachment', 'zafaf_sync_media_on_edit', 10, 1 );
add_filter( 'wp_generate_attachment_metadata', 'zafaf_sync_attachment_sizes_to_minio', 10, 2 );

/**
 * Handle newly uploaded attachments.
 */
function zafaf_sync_media_on_add( int $attachment_id ): void {
    zafaf_sync_media_file_by_id( $attachment_id );
}

/**
 * Handle edited/updated attachments.
 */
function zafaf_sync_media_on_edit( int $attachment_id ): void {
    zafaf_sync_media_file_by_id( $attachment_id );
}

/**
 * Hook to capture all generated sizes (thumbnails) after metadata generation.
 */
function zafaf_sync_attachment_sizes_to_minio( array $metadata, int $attachment_id ): array {
    // Ensure the original attachment file is synced
    zafaf_sync_media_file_by_id( $attachment_id );

    // Sync all generated intermediate sizes
    if ( isset( $metadata['sizes'] ) && is_array( $metadata['sizes'] ) ) {
        $file_path = get_attached_file( $attachment_id );
        if ( $file_path ) {
            $dirname = dirname( $file_path );
            $uploads_dir = wp_get_upload_dir();
            $base_dir = $uploads_dir['basedir'];

            foreach ( $metadata['sizes'] as $size => $info ) {
                if ( isset( $info['file'] ) ) {
                    $sub_file_path = $dirname . '/' . $info['file'];
                    // Get relative path from base uploads directory
                    $relative_path = str_replace( $base_dir, '', $sub_file_path );
                    $mime_type = $info['mime-type'] ?? ( function_exists( 'mime_content_type' ) ? @mime_content_type( $sub_file_path ) : '' ) ?: 'image/jpeg';
                    zafaf_call_rust_media_sync( $relative_path, $mime_type );
                }
            }
        }
    }

    return $metadata;
}

/**
 * Sync the main original attachment file by ID.
 */
function zafaf_sync_media_file_by_id( int $attachment_id ): void {
    $file_path = get_attached_file( $attachment_id );
    if ( ! $file_path || ! file_exists( $file_path ) ) {
        return;
    }

    $uploads_dir = wp_get_upload_dir();
    $base_dir = $uploads_dir['basedir'];

    // Get relative path from wp-content/uploads/
    $relative_path = str_replace( $base_dir, '', $file_path );
    $mime_type = get_post_mime_type( $attachment_id ) ?: ( function_exists( 'mime_content_type' ) ? @mime_content_type( $file_path ) : '' ) ?: 'image/jpeg';

    zafaf_call_rust_media_sync( $relative_path, $mime_type );
}

/**
 * Dispatch sync request to Rust backend.
 */
function zafaf_call_rust_media_sync( string $relative_path, string $mime_type ): void {
    static $synced_paths = array();
    $relative_path = ltrim( $relative_path, '/' );

    // Avoid duplicate requests for the same path in a single execution flow
    if ( isset( $synced_paths[ $relative_path ] ) ) {
        return;
    }
    $synced_paths[ $relative_path ] = true;

    $secret = defined( 'WP_SYNC_SECRET' ) ? WP_SYNC_SECRET : '';
    if ( empty( $secret ) ) {
        error_log( '[ZafafWorld] WP_SYNC_SECRET is not defined — media sync skipped' );
        return;
    }

    $payload = wp_json_encode( array(
        'relative_path' => $relative_path,
        'mime_type'     => $mime_type,
    ) );

    // Send HTTP post to the internal Rust endpoint
    wp_remote_post(
        'http://backend:8080/api/v1/internal/wp-media-sync',
        array(
            'headers'  => array(
                'Content-Type'     => 'application/json',
                'X-Webhook-Secret' => $secret,
            ),
            'body'     => $payload,
            'timeout'  => 15,
            'blocking' => false,
        )
    );
}
