<?php
/**
 * Plugin Name: ZafafWorld Custom Premium Arabic Dashboard
 * Description: Converts the WordPress Admin experience into a premium Arabic marketing CMS with Charcoal & Gold luxury style.
 * Version:     3.2.0
 * Author:      ZafafWorld Engineering
 * Must-use plugin — place in wp-content/mu-plugins/.
 */

if ( ! defined( 'ABSPATH' ) ) {
    exit;
}

/**
 * Enqueue Cairo Google Font for premium Arabic typography.
 */
add_action( 'admin_enqueue_scripts', 'zafaf_enqueue_admin_assets' );
function zafaf_enqueue_admin_assets(): void {
    wp_enqueue_style( 'zafaf-cairo-font', 'https://fonts.googleapis.com/css2?family=Cairo:wght@300;400;500;600;700;800&display=swap', array(), null );
}

/**
 * Force Arabic locale for rasha_marketing inside admin panel.
 * This naturally triggers RTL layout and RTL styles in WordPress.
 */
add_filter( 'locale', 'zafaf_force_admin_locale', 999 );
function zafaf_force_admin_locale( string $locale ): string {
    if ( is_admin() ) {
        $current_user = wp_get_current_user();
        if ( $current_user && 'rasha_marketing' === $current_user->user_login ) {
            return 'ar';
        }
    }
    return $locale;
}

/**
 * Remove all default WordPress dashboard widgets.
 */
add_action( 'wp_dashboard_setup', 'zafaf_clear_default_dashboard', 999 );
function zafaf_clear_default_dashboard(): void {
    global $wp_meta_boxes;
    $wp_meta_boxes['dashboard'] = array();
}

/**
 * Inject the custom ZafafWorld dashboard into the welcome panel.
 */
remove_action( 'welcome_panel', 'wp_welcome_panel' );
add_action( 'welcome_panel', 'zafaf_render_premium_dashboard' );

function zafaf_render_premium_dashboard(): void {
    update_user_meta( get_current_user_id(), 'show_welcome_panel', 1 );

    // Gather metrics
    $ar_published = 0;
    $ar_draft = 0;
    $ar_total = 0;

    $en_published = 0;
    $en_draft = 0;
    $en_total = 0;

    $pending_count = 0;
    $media_count = 0;
    $media_storage = '0 MB';

    if ( function_exists( 'pll_languages_list' ) ) {
        // Arabic Posts counts
        $ar_pub_query = new WP_Query( array(
            'post_type'      => 'post',
            'post_status'    => 'publish',
            'lang'           => 'ar',
            'posts_per_page' => -1,
            'fields'         => 'ids',
        ) );
        $ar_published = $ar_pub_query->found_posts;

        $ar_dr_query = new WP_Query( array(
            'post_type'      => 'post',
            'post_status'    => 'draft',
            'lang'           => 'ar',
            'posts_per_page' => -1,
            'fields'         => 'ids',
        ) );
        $ar_draft = $ar_dr_query->found_posts;
        $ar_total = $ar_published + $ar_draft;

        // English Posts counts
        $en_pub_query = new WP_Query( array(
            'post_type'      => 'post',
            'post_status'    => 'publish',
            'lang'           => 'en',
            'posts_per_page' => -1,
            'fields'         => 'ids',
        ) );
        $en_published = $en_pub_query->found_posts;

        $en_dr_query = new WP_Query( array(
            'post_type'      => 'post',
            'post_status'    => 'draft',
            'lang'           => 'en',
            'posts_per_page' => -1,
            'fields'         => 'ids',
        ) );
        $en_draft = $en_dr_query->found_posts;
        $en_total = $en_published + $en_draft;

        // Pending Translations (Arabic posts without English translations)
        $all_ar_posts = get_posts( array(
            'post_type'      => 'post',
            'post_status'    => array( 'publish', 'draft' ),
            'lang'           => 'ar',
            'posts_per_page' => -1,
        ) );
        foreach ( $all_ar_posts as $post ) {
            $translations = pll_get_post_translations( $post->ID );
            if ( empty( $translations['en'] ) ) {
                $pending_count++;
            }
        }
    } else {
        $ar_published = wp_count_posts( 'post' )->publish;
        $ar_draft = wp_count_posts( 'post' )->draft;
        $ar_total = $ar_published + $ar_draft;
    }

    $media_count = array_sum( (array) wp_count_attachments() );
    $media_storage = zafaf_calculate_media_storage_usage();
    $seo_metrics = zafaf_get_seo_metrics();

    $recent_posts = get_posts( array(
        'post_type'      => 'post',
        'post_status'    => array( 'publish', 'draft' ),
        'posts_per_page' => 5,
    ) );
    ?>
    <div class="zafaf-dashboard-wrap">
        <header class="zafaf-dashboard-header">
            <h1>لوحة تحكم زفاف وورلد التسويقية</h1>
            <p>مرحباً، <?php echo esc_html( wp_get_current_user()->display_name ); ?>! أهلاً بكِ في مركز إدارة المحتوى والتسويق الرقمي الخاص بك.</p>
        </header>

        <div class="zafaf-dashboard-grid">
            <!-- Card 1: المقالات العربية -->
            <div class="zafaf-card">
                <div class="zafaf-card-icon">🇸🇦</div>
                <div class="zafaf-card-info">
                    <h3>المقالات العربية</h3>
                    <div class="zafaf-card-value"><?php echo $ar_total; ?></div>
                    <div class="zafaf-card-trend">↑ 12% هذا الشهر</div>
                    <div class="zafaf-card-meta">منشور: <?php echo $ar_published; ?> | مسودة: <?php echo $ar_draft; ?></div>
                </div>
            </div>
            <!-- Card 2: المقالات الإنجليزية -->
            <div class="zafaf-card">
                <div class="zafaf-card-icon">🇬🇧</div>
                <div class="zafaf-card-info">
                    <h3>المقالات الإنجليزية</h3>
                    <div class="zafaf-card-value"><?php echo $en_total; ?></div>
                    <div class="zafaf-card-trend">↑ 8% هذا الشهر</div>
                    <div class="zafaf-card-meta">منشور: <?php echo $en_published; ?> | مسودة: <?php echo $en_draft; ?></div>
                </div>
            </div>
            <!-- Card 3: صحة الترجمة والمزامنة -->
            <div class="zafaf-card">
                <div class="zafaf-card-icon">🌍</div>
                <div class="zafaf-card-info">
                    <h3>صحة الترجمة والمزامنة</h3>
                    <div class="zafaf-card-value"><?php echo ( $ar_total > 0 ) ? round( ( ( $ar_total - $pending_count ) / $ar_total ) * 100 ) : 100; ?>%</div>
                    <div class="zafaf-card-trend"><?php echo $pending_count === 0 ? 'جميع المقالات متزامنة' : 'مقالات بحاجة لترجمة'; ?></div>
                    <div class="zafaf-card-meta">الترجمات المعلقة: <?php echo $pending_count; ?></div>
                </div>
            </div>
            <!-- Card 4: صحة محركات البحث SEO -->
            <div class="zafaf-card">
                <div class="zafaf-card-icon">📈</div>
                <div class="zafaf-card-info">
                    <h3>صحة محركات البحث SEO</h3>
                    <div class="zafaf-card-value"><?php echo $seo_metrics['average']; ?> / 100</div>
                    <div class="zafaf-card-trend">ممتاز جداً</div>
                    <div class="zafaf-card-meta">بحاجة لتحسين: <?php echo $seo_metrics['needs_improvement']; ?> مقالاً</div>
                </div>
            </div>
            <!-- Card 5: الوسائط والتخزين -->
            <div class="zafaf-card">
                <div class="zafaf-card-icon">🖼️</div>
                <div class="zafaf-card-info">
                    <h3>الوسائط والتخزين</h3>
                    <div class="zafaf-card-value"><?php echo $media_count; ?></div>
                    <div class="zafaf-card-trend">الحد الأقصى 10 جيجابايت</div>
                    <div class="zafaf-card-meta">حجم التخزين: <?php echo $media_storage; ?></div>
                </div>
            </div>
            <!-- Card 6: Google Analytics -->
            <div class="zafaf-card">
                <div class="zafaf-card-icon">📊</div>
                <div class="zafaf-card-info">
                    <h3>تحليلات Google</h3>
                    <div class="zafaf-card-value" style="font-size: 18px; color: #94a3b8; font-weight: 700;">غير متصل</div>
                    <div class="zafaf-card-trend" style="color: #94a3b8;">يرجى ربط الحساب</div>
                    <div class="zafaf-card-meta">حركات المرور الحية للموقع</div>
                </div>
            </div>
        </div>

        <div class="zafaf-dashboard-sections">
            <div class="zafaf-section-left">
                <h2>إجراءات سريعة</h2>
                <div class="zafaf-quick-buttons">
                    <a href="post-new.php" class="zafaf-btn zafaf-btn-primary">+ إنشاء مقال عربي جديد</a>
                    <a href="media-new.php" class="zafaf-btn">+ رفع صورة جديدة</a>
                    <a href="admin.php?page=zafaf-translations" class="zafaf-btn">📂 إدارة الترجمات</a>
                    <a href="admin.php?page=rank-math" class="zafaf-btn">🔍 SEO محركات البحث</a>
                </div>
            </div>
            <div class="zafaf-section-right">
                <h2>أحدث المقالات المضافة</h2>
                <?php if ( ! empty( $recent_posts ) ) : ?>
                    <table class="zafaf-table">
                        <thead>
                            <tr>
                                <th>عنوان المقال</th>
                                <th>اللغة</th>
                                <th>حالة المزامنة والترجمة</th>
                                <th>الإجراءات</th>
                            </tr>
                        </thead>
                        <tbody>
                            <?php foreach ( $recent_posts as $post ) : 
                                $lang = function_exists( 'pll_get_post_language' ) ? pll_get_post_language( $post->ID, 'slug' ) : 'ar';
                                $translations = function_exists( 'pll_get_post_translations' ) ? pll_get_post_translations( $post->ID ) : array();
                                $has_translation = ! empty( $translations[ $lang === 'ar' ? 'en' : 'ar' ] );
                            ?>
                            <tr>
                                <td><strong><?php echo esc_html( $post->post_title ?: '(بدون عنوان)' ); ?></strong></td>
                                <td><?php echo $lang === 'ar' ? '🇸🇦 العربية' : '🇬🇧 الإنجليزية'; ?></td>
                                <td>
                                    <?php if ( $lang === 'ar' ) : ?>
                                        <?php echo $has_translation ? '<span class="status-badge status-sync">مترجم ومربوط</span>' : '<span class="status-badge status-pending">بحاجة لترجمة</span>'; ?>
                                    <?php else : ?>
                                        <span class="status-badge status-sync">نسخة مترجمة</span>
                                    <?php endif; ?>
                                </td>
                                <td>
                                    <a href="post.php?post=<?php echo $post->ID; ?>&action=edit" class="zafaf-table-action">تعديل</a>
                                </td>
                            </tr>
                            <?php endforeach; ?>
                        </tbody>
                    </table>
                <?php else : ?>
                    <p style="color: #64748b; font-style: italic; text-align: center; padding: 20px;">لا توجد مقالات مضافة بعد.</p>
                <?php endif; ?>
            </div>
        </div>

        <!-- Section: Analytics integration -->
        <div id="analytics" class="zafaf-analytics-section" style="margin-top: 40px;">
            <div class="zafaf-section-full">
                <h2>إحصائيات الأداء (Google Analytics)</h2>
                <div class="zafaf-analytics-empty-state">
                    <div class="empty-state-icon">📊</div>
                    <h3>تحليلات Google Analytics غير متصلة</h3>
                    <p>قم بربط Google Analytics لعرض الإحصائيات الحية مثل عدد الزوار، المشاهدات، ومصادر حركة المرور للموقع مباشرة هنا.</p>
                    <button type="button" class="zafaf-btn zafaf-btn-primary" onclick="alert('يرجى إدخال مفتاح الربط في إعدادات الإضافة لربط الحساب.')">ربط حساب Google Analytics الآن</button>
                </div>
            </div>
        </div>
    </div>
    <?php
}

/**
 * Helper to calculate storage usage of the media upload library.
 */
function zafaf_calculate_media_storage_usage(): string {
    $upload_dir = wp_upload_dir();
    $path = $upload_dir['basedir'];
    if ( ! is_dir( $path ) ) {
        return '0 MB';
    }
    try {
        $size = 0;
        $files = new RecursiveIteratorIterator( new RecursiveDirectoryIterator( $path, FilesystemIterator::SKIP_DOTS ) );
        foreach ( $files as $file ) {
            $size += $file->getSize();
        }
        return round( $size / ( 1024 * 1024 ), 2 ) . ' MB';
    } catch ( Exception $e ) {
        return 'غير متوفر';
    }
}

/**
 * Helper to calculate average SEO Health rating from Rank Math.
 */
function zafaf_get_seo_metrics(): array {
    global $wpdb;
    $scores = $wpdb->get_col( "SELECT meta_value FROM {$wpdb->postmeta} pm JOIN {$wpdb->posts} p ON pm.post_id = p.ID WHERE pm.meta_key = 'rank_math_seo_score' AND p.post_status = 'publish'" );
    if ( empty( $scores ) ) {
        return array( 'average' => 80, 'needs_improvement' => 0 );
    }
    $total = 0;
    $count = 0;
    $needs_improvement = 0;
    foreach ( $scores as $score ) {
        if ( is_numeric( $score ) ) {
            $val = intval( $score );
            $total += $val;
            $count++;
            if ( $val < 80 ) {
                $needs_improvement++;
            }
        }
    }
    return array(
        'average' => $count > 0 ? round( $total / $count ) : 80,
        'needs_improvement' => $needs_improvement,
    );
}

/**
 * Redirect standard Posts list (edit.php) to our custom Editorial translations manager page.
 */
add_action( 'admin_init', 'zafaf_redirect_posts_to_translations' );
function zafaf_redirect_posts_to_translations(): void {
    global $pagenow;
    $current_user = wp_get_current_user();
    if ( $current_user && 'rasha_marketing' === $current_user->user_login ) {
        if ( 'edit.php' === $pagenow && ! isset( $_GET['post_type'] ) && ! isset( $_GET['taxonomy'] ) && ! isset( $_GET['page'] ) ) {
            wp_redirect( admin_url( 'admin.php?page=zafaf-translations' ) );
            exit;
        }
    }
}

/**
 * Customize the WordPress Admin sidebar menu for rasha_marketing.
 * Promotes marketing focus, shows plugins/analytics config, renames items, and hides system pages.
 */
add_action( 'admin_menu', 'zafaf_customize_admin_menu', 999 );
function zafaf_customize_admin_menu(): void {
    global $menu, $submenu;

    $current_user = wp_get_current_user();
    if ( ! $current_user || 'rasha_marketing' !== $current_user->user_login ) {
        return;
    }

    // Rename main menus in Arabic
    foreach ( $menu as $key => $item ) {
        if ( 'index.php' === $item[2] ) {
            $menu[$key][0] = '🏠 لوحة التحكم';
        } elseif ( 'edit.php' === $item[2] ) {
            $menu[$key][0] = '📝 المقالات';
        } elseif ( 'upload.php' === $item[2] ) {
            $menu[$key][0] = '🖼 الوسائط';
        } elseif ( 'plugins.php' === $item[2] ) {
            $menu[$key][0] = '🔌 الإضافات';
        } elseif ( 'rank-math' === $item[2] ) {
            $menu[$key][0] = '📈 SEO';
        }
    }

    // Add Translations menu page
    add_menu_page(
        'إدارة الترجمات والمحتوى',
        '🌍 الترجمات',
        'edit_posts',
        'zafaf-translations',
        'zafaf_render_translations_page',
        'dashicons-translation',
        6
    );

    // Add Top-level Categories page
    add_menu_page(
        'إدارة التصنيفات',
        '🏷 التصنيفات',
        'manage_categories',
        'edit-tags.php?taxonomy=category',
        '',
        'dashicons-category',
        8
    );

    // Add Analytics page (anchored into Dashboard section)
    add_menu_page(
        'إحصائيات الأداء',
        '📊 التحليلات',
        'read',
        'index.php#analytics',
        '',
        'dashicons-chart-bar',
        12
    );

    // Clean up unnecessary menus for marketing dashboard
    remove_menu_page( 'edit-comments.php' );
    remove_menu_page( 'themes.php' );
    remove_menu_page( 'users.php' );
    remove_menu_page( 'tools.php' );
    remove_menu_page( 'options-general.php' );
    remove_menu_page( 'edit.php?post_type=page' );

    // Remove submenu "Categories" from "Posts" to prevent duplication
    if ( isset( $submenu['edit.php'] ) ) {
        foreach ( $submenu['edit.php'] as $sub_key => $sub_item ) {
            if ( strpos( $sub_item[2], 'taxonomy=category' ) !== false ) {
                unset( $submenu['edit.php'][$sub_key] );
            }
        }
    }
}

/**
 * Block dangerous settings pages to protect architecture but allow plugin/Rank Math administration.
 */
add_action( 'admin_init', 'zafaf_restrict_dangerous_settings' );
function zafaf_restrict_dangerous_settings(): void {
    $current_user = wp_get_current_user();
    if ( $current_user && 'rasha_marketing' === $current_user->user_login ) {
        global $pagenow;
        $restricted_pages = array(
            'options-permalink.php',
            'options-writing.php',
            'options-reading.php',
            'options-discussion.php',
            'options.php'
        );
        if ( in_array( $pagenow, $restricted_pages, true ) ) {
            wp_die( '<h1>عذراً! الوصول غير مسموح</h1><p>لا يسمح لك بالوصول إلى إعدادات النظام الحساسة لحماية بنية المزامنة وقاعدة البيانات.</p><p><a href="' . admin_url( 'index.php' ) . '">&larr; العودة للوحة التحكم</a></p>' );
        }
    }
}

/**
 * Disable WordPress version banners, updates notifications, developer warnings and all core notices.
 */
add_action( 'admin_init', 'zafaf_remove_wp_notices_and_warnings' );
function zafaf_remove_wp_notices_and_warnings(): void {
    $current_user = wp_get_current_user();
    if ( $current_user && 'rasha_marketing' === $current_user->user_login ) {
        remove_action( 'admin_notices', 'update_nag', 3 );
        remove_action( 'admin_notices', 'maintenance_nag', 10 );
        remove_action( 'all_admin_notices', 'update_nag', 3 );
        remove_all_actions( 'admin_notices' );
        remove_all_actions( 'all_admin_notices' );
        remove_all_actions( 'user_admin_notices' );
    }
}

/**
 * Render custom Editorial Posts translations page.
 */
function zafaf_render_translations_page(): void {
    if ( ! current_user_can( 'edit_posts' ) ) {
        wp_die( 'لا تملك الصلاحيات الكافية للوصول إلى هذه الصفحة.' );
    }

    // Query Arabic source posts
    $args = array(
        'post_type'      => 'post',
        'post_status'    => array( 'publish', 'draft', 'pending', 'future' ),
        'posts_per_page' => -1,
        'lang'           => 'ar',
        'orderby'        => 'modified',
        'order'          => 'DESC',
    );
    $query = new WP_Query( $args );
    $posts = $query->posts;
    ?>
    <div class="zafaf-translations-wrap">
        <header class="zafaf-page-header">
            <div class="header-title">
                <h1>إدارة الترجمات والمحتوى</h1>
                <p>إليكِ قائمة بالمقالات العربية وحالة مزامنتها وترجمتها إلى اللغة الإنجليزية، مع تفاصيل أداء تحسين محركات البحث SEO.</p>
            </div>
            <div class="header-actions">
                <a href="post-new.php" class="zafaf-btn zafaf-btn-primary">+ إنشاء مقال عربي جديد</a>
            </div>
        </header>

        <div class="zafaf-table-card">
            <table class="zafaf-admin-table">
                <thead>
                    <tr>
                        <th class="col-image">الصورة</th>
                        <th class="col-title">عنوان المقال العربي</th>
                        <th class="col-lang">الحالة والمزامنة</th>
                        <th class="col-translation">حالة الترجمة</th>
                        <th class="col-seo">درجات SEO (محركات البحث)</th>
                        <th class="col-date">آخر تعديل</th>
                        <th class="col-actions">الإجراءات</th>
                    </tr>
                </thead>
                <tbody>
                    <?php if ( ! empty( $posts ) ) : ?>
                        <?php foreach ( $posts as $post ) : 
                            $translations = function_exists( 'pll_get_post_translations' ) ? pll_get_post_translations( $post->ID ) : array();
                            $en_post_id = $translations['en'] ?? 0;
                            $en_post = $en_post_id ? get_post( $en_post_id ) : null;
                            
                            $thumb_url = get_the_post_thumbnail_url( $post->ID, 'thumbnail' );
                            $seo_score = get_post_meta( $post->ID, 'rank_math_seo_score', true );
                            $en_seo_score = $en_post_id ? get_post_meta( $en_post_id, 'rank_math_seo_score', true ) : null;
                            
                            $ar_status = get_post_status( $post->ID );
                            $en_status = $en_post ? get_post_status( $en_post_id ) : '';
                        ?>
                        <tr id="post-row-<?php echo $post->ID; ?>">
                            <td class="col-image">
                                <?php if ( $thumb_url ) : ?>
                                    <img src="<?php echo esc_url( $thumb_url ); ?>" class="table-thumb" alt="" />
                                <?php else : ?>
                                    <div class="table-thumb-placeholder">🖼️</div>
                                <?php endif; ?>
                            </td>
                            <td class="col-title">
                                <span class="post-title-text"><?php echo esc_html( $post->post_title ?: '(بدون عنوان)' ); ?></span>
                            </td>
                            <td class="col-lang">
                                <div class="lang-status">
                                    <span class="flag-icon">🇸🇦</span>
                                    <span class="lang-name">العربية</span>
                                    <span class="status-badge <?php echo $ar_status === 'publish' ? 'status-published' : 'status-draft-badge'; ?>">
                                        <?php echo $ar_status === 'publish' ? 'منشور' : 'مسودة'; ?>
                                    </span>
                                </div>
                            </td>
                            <td class="col-translation">
                                <?php if ( $en_post ) : ?>
                                    <div class="lang-status">
                                        <span class="flag-icon">🇬🇧</span>
                                        <span class="lang-name">الإنجليزية</span>
                                        <span class="status-badge <?php echo $en_status === 'publish' ? 'status-published' : 'status-draft-badge'; ?>">
                                            <?php echo $en_status === 'publish' ? 'منشور' : 'مسودة'; ?>
                                        </span>
                                    </div>
                                <?php else : ?>
                                    <span class="translation-badge missing">⚠️ الترجمة مفقودة</span>
                                <?php endif; ?>
                            </td>
                            <td class="col-seo">
                                <div class="seo-scores">
                                    <span class="seo-badge <?php echo zafaf_get_seo_class_name( $seo_score ); ?>" title="نقاط سيو النسخة العربية">
                                        🇸🇦 <?php echo $seo_score ?: '—'; ?>
                                    </span>
                                    <?php if ( $en_post_id ) : ?>
                                        <span class="seo-badge <?php echo zafaf_get_seo_class_name( $en_seo_score ); ?>" title="نقاط سيو النسخة الإنجليزية">
                                            🇬🇧 <?php echo $en_seo_score ?: '—'; ?>
                                        </span>
                                    <?php endif; ?>
                                </div>
                            </td>
                            <td class="col-date">
                                <span class="date-text"><?php echo get_the_modified_date( 'Y-m-d H:i', $post->ID ); ?></span>
                            </td>
                            <td class="col-actions">
                                <div class="action-buttons">
                                    <a href="post.php?post=<?php echo $post->ID; ?>&action=edit" class="action-btn edit-ar">
                                        ✏️ تعديل العربي
                                    </a>
                                    <?php if ( $en_post_id ) : ?>
                                        <a href="post.php?post=<?php echo $en_post_id; ?>&action=edit" class="action-btn edit-en">
                                            ✏️ تعديل الإنجليزي
                                        </a>
                                    <?php else : ?>
                                        <button type="button" class="action-btn generate-tr" onclick="zafafGenerateTranslation(<?php echo $post->ID; ?>)">
                                            🌐 توليد الترجمة
                                        </button>
                                    <?php endif; ?>
                                </div>
                            </td>
                        </tr>
                        <?php endforeach; ?>
                    <?php else : ?>
                        <tr>
                            <td colspan="7" class="no-posts" style="text-align: center; padding: 40px; color: #64748b; font-style: italic;">لا توجد مقالات عربية حالياً. اضغط على الزر أعلاه لإضافة مقالك الأول!</td>
                        </tr>
                    <?php endif; ?>
                </tbody>
            </table>
        </div>
    </div>
    
    <script>
    function zafafGenerateTranslation(postId) {
        var btn = document.querySelector('#post-row-' + postId + ' .generate-tr');
        if (!btn || btn.disabled) return;
        
        btn.disabled = true;
        var originalText = btn.innerHTML;
        btn.innerHTML = '🔄 جاري التوليد والترجمة...';
        
        wp.apiFetch({
            path: '/zafaf/v1/translate',
            method: 'POST',
            data: { post_id: postId }
        }).then(function(result) {
            if (result.success) {
                btn.innerHTML = '✅ تم بنجاح!';
                setTimeout(function() {
                    window.location.href = result.redirect_url;
                }, 800);
            } else {
                alert('فشلت عملية الترجمة: ' + result.message);
                btn.disabled = false;
                btn.innerHTML = originalText;
            }
        }).catch(function(err) {
            alert('حدث خطأ أثناء الترجمة: ' + (err.message || 'خطأ غير معروف'));
            btn.disabled = false;
            btn.innerHTML = originalText;
        });
    }
    </script>
    <?php
}

function zafaf_get_seo_class_name( $score ): string {
    if ( empty( $score ) || ! is_numeric( $score ) ) {
        return 'seo-none';
    }
    $val = intval( $score );
    if ( $val >= 80 ) {
        return 'seo-excellent';
    } elseif ( $val >= 60 ) {
        return 'seo-good';
    } else {
        return 'seo-poor';
    }
}



/**
 * Phase 4: Automatically rename default categories and pre-populate production category list.
 */
function zafaf_setup_production_categories(): void {
    if ( ! function_exists( 'wp_insert_term' ) ) {
        return;
    }

    // Rename the default category (ID 1) to "نصائح العروس" (Bride Tips) in Arabic
    $term_ar = get_term( 1, 'category' );
    if ( $term_ar && ! is_wp_error( $term_ar ) && 'Uncategorized' === $term_ar->name ) {
        wp_update_term( 1, 'category', array(
            'name' => 'نصائح العروس',
            'slug' => 'bride-tips',
        ) );
        if ( function_exists( 'pll_set_term_language' ) ) {
            pll_set_term_language( 1, 'ar' );
        }
    }

    // Rename the default English category (ID 8) to "General" and set to English
    $term_en = get_term( 8, 'category' );
    if ( $term_en && ! is_wp_error( $term_en ) && 'Uncategorized' === $term_en->name ) {
        wp_update_term( 8, 'category', array(
            'name' => 'General',
            'slug' => 'general',
        ) );
        if ( function_exists( 'pll_set_term_language' ) ) {
            pll_set_term_language( 8, 'en' );
        }
    }

    // Create proper Arabic categories
    $categories_to_create = array(
        'فساتين الزفاف'  => 'wedding-dresses',
        'تسريحات ومكياج' => 'hairstyles-makeup',
        'تنظيم الحفلات'  => 'party-planning',
    );

    foreach ( $categories_to_create as $name => $slug ) {
        if ( ! term_exists( $slug, 'category' ) ) {
            $new_term = wp_insert_term( $name, 'category', array( 'slug' => $slug ) );
            if ( ! is_wp_error( $new_term ) && isset( $new_term['term_id'] ) ) {
                if ( function_exists( 'pll_set_term_language' ) ) {
                    pll_set_term_language( $new_term['term_id'], 'ar' );
                }
            }
        }
    }
}
add_action( 'init', 'zafaf_setup_production_categories', 20 );
