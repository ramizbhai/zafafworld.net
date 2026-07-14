<script lang="ts">
    import { getStatusLabel, getStatusBadgeClass, formatInquiryDate } from "../../services/inquiries.service";

    let { inquiry, i18n, inquiriesState } = $props<{ inquiry: any, i18n: any, inquiriesState: any }>();

    function openInquiryDetail() {
        inquiriesState.selectedInquiry = inquiry;
        inquiriesState.isDrawerOpen = true;
    }
</script>

<button 
    type="button" 
    onclick={openInquiryDetail} 
    class="lead-card"
    class:selected-card={inquiriesState.activeSelectedInquiry?.id === inquiry.id}
>
    <div class="card-header">
        <div class="user-block">
            <div class="avatar" class:pending-avatar={inquiry.status === 'pending'}>
                {(inquiry.client_first_name || inquiry.client_email || inquiry.name || inquiry.email || 'G').charAt(0).toUpperCase()}
            </div>
            <div class="info">
                <div class="name-row">
                    <span class="customer-name">
                        {inquiry.client_first_name ? `${inquiry.client_first_name} ${inquiry.client_last_name || ''}` : (inquiry.name || inquiry.email || (i18n.locale === 'ar' ? 'عميل زائر' : 'Guest Client'))}
                    </span>
                </div>
                <span class="receipt-time">
                    {i18n.locale === 'ar' ? 'تاريخ الاستلام:' : 'Received on'} {formatInquiryDate(inquiry.created_at, i18n.locale)}
                </span>
            </div>
        </div>

        <span class="status-badge {getStatusBadgeClass(inquiry.status)}">
            {getStatusLabel(inquiry.status, i18n)}
        </span>
    </div>

    <div class="card-body">
        <p class="message-snippet">{inquiry.message}</p>
    </div>

    <div class="card-footer">
        <div class="footer-pill">
            📞 {inquiry.client_phone || inquiry.phone || (i18n.locale === 'ar' ? 'لا يوجد هاتف' : 'No phone')}
        </div>
        <div class="footer-pill">
            📅 {i18n.locale === 'ar' ? 'تاريخ الحفل:' : 'Event Date:'} {inquiry.event_date}
        </div>
        <div class="footer-pill">
            👥 {i18n.locale === 'ar' ? 'الضيوف:' : 'Guests:'} {inquiry.guest_count}
        </div>
    </div>
</button>
