(function(wp) {
    if (!wp || !wp.element) return;
    
    var useState = wp.element.useState;
    var useEffect = wp.element.useEffect;
    var el = wp.element.createElement;

    // Inject ZafafWorld premium styling and sidebar cleanup CSS
    var style = document.createElement('style');
    style.innerHTML = `
        .zafaf-editor-tabs-container {
            display: inline-flex;
            align-items: center;
            margin-left: 10px;
            margin-right: 10px;
            gap: 8px;
            font-family: 'Cairo', sans-serif;
        }
        .zafaf-editor-tabs {
            display: flex;
            align-items: center;
            background: #141414; /* Charcoal Black */
            border-radius: 20px;
            padding: 3px;
            border: 1.5px solid #C8A45D; /* Soft Gold */
        }
        .zafaf-editor-tab {
            padding: 4px 12px;
            border-radius: 18px;
            font-size: 12px;
            font-weight: 700;
            cursor: pointer;
            border: none;
            background: transparent;
            color: #B8B2A7; /* Warm grey inactive */
            transition: all 0.2s ease-in-out;
            display: flex;
            align-items: center;
            gap: 4px;
        }
        .zafaf-editor-tab.active {
            background: #C8A45D; /* Soft Gold */
            color: #1C1A17; /* Dark charcoal text */
            box-shadow: 0 2px 6px rgba(200, 164, 93, 0.4);
        }
        .zafaf-editor-tab:hover:not(.active) {
            color: #F5F1E8; /* Warm white hover */
        }
        .zafaf-editor-btn-generate {
            padding: 4px 12px;
            border-radius: 18px;
            font-size: 12px;
            font-weight: 800;
            cursor: pointer;
            border: 1.5px solid #C8A45D;
            background: transparent;
            color: #C8A45D;
            transition: all 0.2s ease;
            display: flex;
            align-items: center;
            gap: 4px;
        }
        .zafaf-editor-btn-generate:hover:not(:disabled) {
            background: #C8A45D;
            color: #1C1A17;
            box-shadow: 0 2px 6px rgba(200, 164, 93, 0.3);
        }
        .zafaf-editor-btn-generate:disabled {
            opacity: 0.6;
            cursor: not-allowed;
        }
        .zafaf-premium-modal {
            max-width: 500px;
            font-family: 'Cairo', sans-serif;
            border: 1px solid #C8A45D;
            border-radius: 12px;
        }
        .zafaf-premium-modal .components-modal__header {
            background: #141414; /* Charcoal Black */
            color: #F5F1E8;
            border-bottom: 2px solid #C8A45D;
            padding: 15px 20px;
        }
        .zafaf-premium-modal .components-modal__header h2 {
            color: #C8A45D !important;
            font-weight: 800;
        }
        .zafaf-premium-modal .components-modal__content {
            padding: 20px;
            background: #FFFEFA;
        }
        .zafaf-modal-buttons {
            display: flex;
            gap: 10px;
            justify-content: flex-end;
            margin-top: 25px;
        }
        .zafaf-modal-btn {
            padding: 8px 18px !important;
            border-radius: 8px !important;
            font-weight: 700 !important;
            font-family: 'Cairo', sans-serif !important;
        }
        .zafaf-modal-btn-primary {
            background: #C8A45D !important;
            color: #1C1A17 !important;
            border: 1px solid #C8A45D !important;
        }
        .zafaf-modal-btn-primary:hover {
            background: #b59350 !important;
        }
        .zafaf-modal-btn-secondary {
            background: #FFFEFA !important;
            color: #1C1A17 !important;
            border: 1px solid #C8A45D !important;
        }
        .zafaf-modal-btn-secondary:hover {
            background: #FAF8F3 !important;
        }

        /* ── Sidebar Polish: Hide unnecessary technical fields ── */
        .components-panel__body.editor-post-discussion,
        .components-panel__body:has(.editor-post-discussion) {
            display: none !important;
        }
        .components-panel__body.editor-page-attributes,
        .components-panel__body:has(.editor-page-attributes),
        .components-panel__body:has(.editor-post-template),
        .editor-post-template {
            display: none !important;
        }
        .components-panel__body:has(.editor-post-link),
        .editor-post-link {
            display: none !important;
        }
        .editor-post-author,
        .editor-post-format {
            display: none !important;
        }
    `;
    document.head.appendChild(style);

    var ZafafTabsComponent = function() {
        var [isTranslating, setIsTranslating] = useState(false);
        var [status, setStatus] = useState('');
        var [isModalOpen, setIsModalOpen] = useState(false);
        var [isUpdateModalOpen, setIsUpdateModalOpen] = useState(false);

        var settings = window.zafafTranslationSettings || {};
        var [postId, setPostId] = useState(settings.post_id || null);
        var [currentLang, setCurrentLang] = useState(settings.current_lang || 'ar');
        var [translations, setTranslations] = useState(settings.translations || {});

        // Periodically sync post ID from Gutenberg store
        useEffect(function() {
            var interval = setInterval(function() {
                var store = wp.data.select('core/editor');
                if (store) {
                    var id = store.getCurrentPostId();
                    if (id && id !== postId) {
                        setPostId(id);
                        wp.apiFetch({
                            path: '/zafaf/v1/status?post_id=' + id
                        }).then(function(result) {
                            if (result.success) {
                                setCurrentLang(result.current_lang);
                                setTranslations(result.translations || {});
                            }
                        }).catch(function(err) {});
                    }
                }
            }, 1000);
            return function() {
                clearInterval(interval);
            };
        }, [postId]);

        // Direction alignment for editor body, sidebar, & iframes
        useEffect(function() {
            var direction = (currentLang === 'ar') ? 'rtl' : 'ltr';
            function applyDir() {
                var html = document.documentElement;
                if (html.getAttribute('dir') !== direction) {
                    html.setAttribute('dir', direction);
                    html.style.direction = direction;
                }
                
                if (direction === 'rtl') {
                    document.body.classList.add('rtl');
                    document.body.classList.remove('ltr');
                } else {
                    document.body.classList.add('ltr');
                    document.body.classList.remove('rtl');
                }

                var selectors = [
                    '.editor-styles-wrapper',
                    '.block-editor-writing-flow',
                    '.block-editor-iframe__body',
                    '.interface-interface-skeleton__sidebar'
                ];
                selectors.forEach(function(sel) {
                    var elms = document.querySelectorAll(sel);
                    elms.forEach(function(elm) {
                        if (elm.getAttribute('dir') !== direction) {
                            elm.setAttribute('dir', direction);
                            elm.style.direction = direction;
                        }
                    });
                });
                
                var iframes = document.querySelectorAll('iframe.block-editor-iframe__iframe');
                iframes.forEach(function(iframe) {
                    try {
                        var doc = iframe.contentDocument || iframe.contentWindow.document;
                        if (doc) {
                            var body = doc.querySelector('.block-editor-iframe__body') || doc.body;
                            if (body && body.getAttribute('dir') !== direction) {
                                body.setAttribute('dir', direction);
                                body.style.direction = direction;
                            }
                            var iframeHtml = doc.documentElement;
                            if (iframeHtml && iframeHtml.getAttribute('dir') !== direction) {
                                iframeHtml.setAttribute('dir', direction);
                                iframeHtml.style.direction = direction;
                            }
                        }
                    } catch (e) {}
                });
            }
            applyDir();
            var interval = setInterval(applyDir, 500);
            return function() {
                clearInterval(interval);
            };
        }, [currentLang]);

        // Intercept Post Update (Phase 4: Ask user to update translation also)
        useEffect(function() {
            var wasSaving = false;
            var unsubscribe = wp.data.subscribe(function() {
                var store = wp.data.select('core/editor');
                if (!store) return;
                
                var isSaving = store.isSavingPost();
                var isAutosaving = store.isAutosavingPost();
                var isSaveSuccess = store.didPostSaveRequestSucceed();
                
                if (isSaving && !isAutosaving) {
                    wasSaving = true;
                }
                
                if (wasSaving && !isSaving) {
                    wasSaving = false;
                    if (isSaveSuccess) {
                        var hasOpposing = (currentLang === 'ar') ? !!(translations && translations.en) : !!(translations && translations.ar);
                        if (hasOpposing) {
                            setIsUpdateModalOpen(true);
                        }
                    }
                }
            });
            return function() {
                unsubscribe();
            };
        }, [currentLang, translations]);

        var hasEnglish = !!(translations && translations.en);
        var hasArabic = !!(translations && translations.ar);

        var onTabClick = function(targetLang) {
            if (targetLang === currentLang) return;

            var targetPostId = translations[targetLang];
            if (targetPostId) {
                window.location.href = 'post.php?post=' + targetPostId + '&action=edit';
            } else {
                setIsModalOpen(true);
            }
        };

        var onTranslateClick = function() {
            if (isTranslating) return;
            if (!postId) {
                alert(currentLang === 'ar' ? 'يرجى حفظ المسودة أولاً لتفعيل الترجمة.' : 'Please save draft first to activate translation.');
                return;
            }
            setIsTranslating(true);
            setStatus(currentLang === 'ar' ? 'جاري التوليد...' : 'Translating...');

            wp.apiFetch({
                path: '/zafaf/v1/translate',
                method: 'POST',
                data: { post_id: postId }
            }).then(function(result) {
                setIsTranslating(false);
                setIsModalOpen(false);
                setIsUpdateModalOpen(false);
                if (result.success) {
                    setStatus(currentLang === 'ar' ? 'تم بنجاح!' : 'Success!');
                    setTimeout(function() {
                        window.location.href = result.redirect_url;
                    }, 800);
                } else {
                    setStatus('');
                    alert('Error: ' + result.message);
                }
            }).catch(function(err) {
                setIsTranslating(false);
                setStatus('');
                alert('Error: ' + (err.message || 'Unknown error.'));
            });
        };

        var arLabel = 'العربية 🇸🇦';
        var enLabel = 'English 🇬🇧';
        var isAr = (currentLang === 'ar');
        
        var modalTitle = isAr ? 'النسخة الإنجليزية غير موجودة' : 'Arabic Version Missing';
        var modalText = isAr 
            ? 'النسخة الإنجليزية غير موجودة لهذا المقال بعد. هل تريد إنشاء وترجمة هذا المقال الآن؟' 
            : 'The Arabic version is not created yet. Generate translation now?';
        var modalBtnLabel = isAr ? 'إنشاء الترجمة الآن' : 'Generate Translation';
        var modalCloseLabel = isAr ? 'إلغاء' : 'Cancel';

        var updateModalTitle = isAr ? 'تحديث الترجمة الإنجليزية' : 'Update Arabic Translation';
        var updateModalText = isAr
            ? 'هل تريد تحديث الترجمة الإنجليزية أيضاً لمزامنة التغييرات التي قمت بها مع النسخة الإنجليزية؟'
            : 'Do you want to update the Arabic translation also to sync changes?';
        var updateBtnLabel = isAr ? 'تحديث الترجمة 🔄' : 'Update Translation';
        var onlyCurrentBtnLabel = isAr ? 'فقط العربية' : 'Only English';

        var translateBtnLabel = isAr ? 'Translate to English 🇬🇧' : 'Translate to Arabic 🇸🇦';

        return el(
            'div',
            { className: 'zafaf-editor-tabs-container' },
            el(
                'div',
                { className: 'zafaf-editor-tabs' },
                el(
                    'button',
                    {
                        className: 'zafaf-editor-tab' + (isAr ? ' active' : ''),
                        onClick: function() { onTabClick('ar'); },
                        type: 'button'
                    },
                    arLabel
                ),
                el(
                    'button',
                    {
                        className: 'zafaf-editor-tab' + (!isAr ? ' active' : ''),
                        onClick: function() { onTabClick('en'); },
                        type: 'button'
                    },
                    enLabel
                )
            ),
            el(
                'button',
                {
                    className: 'zafaf-editor-btn-generate',
                    disabled: isTranslating,
                    onClick: onTranslateClick,
                    type: 'button'
                },
                isTranslating ? status : translateBtnLabel
            ),
            
            // Creation Modal
            isModalOpen && el(
                wp.components.Modal,
                {
                    title: modalTitle,
                    onRequestClose: function() { setIsModalOpen(false); },
                    className: 'zafaf-premium-modal'
                },
                el('p', { style: { marginBottom: '20px', fontSize: '15px', lineHeight: '1.6', color: '#6F675C' } }, modalText),
                el(
                    'div',
                    { className: 'zafaf-modal-buttons' },
                    el(
                        wp.components.Button,
                        {
                            className: 'zafaf-modal-btn zafaf-modal-btn-secondary',
                            onClick: function() { setIsModalOpen(false); }
                        },
                        modalCloseLabel
                    ),
                    el(
                        wp.components.Button,
                        {
                            className: 'zafaf-modal-btn zafaf-modal-btn-primary',
                            disabled: isTranslating,
                            onClick: onTranslateClick
                        },
                        isTranslating ? status : modalBtnLabel
                    )
                )
            ),

            // Update Modal
            isUpdateModalOpen && el(
                wp.components.Modal,
                {
                    title: updateModalTitle,
                    onRequestClose: function() { setIsUpdateModalOpen(false); },
                    className: 'zafaf-premium-modal'
                },
                el('p', { style: { marginBottom: '20px', fontSize: '15px', lineHeight: '1.6', color: '#6F675C' } }, updateModalText),
                el(
                    'div',
                    { className: 'zafaf-modal-buttons' },
                    el(
                        wp.components.Button,
                        {
                            className: 'zafaf-modal-btn zafaf-modal-btn-secondary',
                            onClick: function() { setIsUpdateModalOpen(false); }
                        },
                        onlyCurrentBtnLabel
                    ),
                    el(
                        wp.components.Button,
                        {
                            className: 'zafaf-modal-btn zafaf-modal-btn-primary',
                            disabled: isTranslating,
                            onClick: onTranslateClick
                        },
                        isTranslating ? status : updateBtnLabel
                    )
                )
            )
        );
    };

    // ── Persistent DOM-injected React toolbar ──
    function injectToolbar() {
        var target = document.querySelector('.edit-post-header__settings');
        if (!target) {
            target = document.querySelector('.edit-post-header-toolbar');
        }
        if (!target) return;

        var wrapper = document.getElementById('zafaf-header-toolbar-wrapper');
        if (!wrapper) {
            wrapper = document.createElement('div');
            wrapper.id = 'zafaf-header-toolbar-wrapper';
            wrapper.style.display = 'inline-flex';
            wrapper.style.alignItems = 'center';
            
            if (target.classList.contains('edit-post-header__settings')) {
                target.insertBefore(wrapper, target.firstChild);
            } else {
                target.appendChild(wrapper);
            }
        }

        wp.element.render(
            el(ZafafTabsComponent),
            wrapper
        );
    }

    injectToolbar();
    var injectInterval = setInterval(injectToolbar, 800);
})(window.wp);
