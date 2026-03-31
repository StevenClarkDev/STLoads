{{-- Toast Notification Container
    Include this once in your layout file
    Usage in JavaScript:
    showToast('Success!', 'Your changes have been saved.', 'success');
    showToast('Error!', 'Something went wrong.', 'error');
    showToast('Warning!', 'Please check your input.', 'warning');
    showToast('Info', 'New update available.', 'info');
--}}

<div id="toast-container" class="toast-container position-fixed top-0 end-0 p-3" style="z-index: 9999;"></div>

@once
@push('styles')
<style>
    .toast-container {
        max-width: 380px;
    }
    
    .toast {
        border-left: 4px solid;
        box-shadow: 0 4px 12px rgba(0,0,0,0.15);
    }
    
    .toast.toast-success {
        border-left-color: #28a745;
    }
    
    .toast.toast-error {
        border-left-color: #dc3545;
    }
    
    .toast.toast-warning {
        border-left-color: #ffc107;
    }
    
    .toast.toast-info {
        border-left-color: #17a2b8;
    }
    
    .toast-header {
        font-weight: 600;
    }
    
    .toast-icon {
        width: 20px;
        height: 20px;
        margin-right: 8px;
    }
    
    .toast-success .toast-icon { color: #28a745; }
    .toast-error .toast-icon { color: #dc3545; }
    .toast-warning .toast-icon { color: #ffc107; }
    .toast-info .toast-icon { color: #17a2b8; }
</style>
@endpush

@push('scripts')
<script>
/**
 * Show a toast notification
 * @param {string} title - Toast title
 * @param {string} message - Toast message
 * @param {string} type - Toast type: 'success', 'error', 'warning', 'info'
 * @param {number} duration - Auto-hide duration in ms (0 = no auto-hide)
 */
function showToast(title, message, type = 'info', duration = 5000) {
    const container = document.getElementById('toast-container');
    if (!container) return;
    
    // Icon mapping
    const icons = {
        success: 'check-circle',
        error: 'x-circle',
        warning: 'alert-triangle',
        info: 'info'
    };
    
    // Create unique ID
    const toastId = 'toast-' + Date.now();
    
    // Create toast HTML
    const toastHTML = `
        <div id="${toastId}" class="toast toast-${type}" role="alert" aria-live="assertive" aria-atomic="true">
            <div class="toast-header">
                <i data-feather="${icons[type] || 'info'}" class="toast-icon"></i>
                <strong class="me-auto">${title}</strong>
                <small class="text-muted">Just now</small>
                <button type="button" class="btn-close" data-bs-dismiss="toast" aria-label="Close"></button>
            </div>
            <div class="toast-body">
                ${message}
            </div>
        </div>
    `;
    
    // Add toast to container
    container.insertAdjacentHTML('beforeend', toastHTML);
    
    // Get toast element
    const toastElement = document.getElementById(toastId);
    
    // Initialize feather icons
    if (typeof feather !== 'undefined') {
        feather.replace();
    }
    
    // Initialize Bootstrap toast
    const toast = new bootstrap.Toast(toastElement, {
        autohide: duration > 0,
        delay: duration
    });
    
    // Show toast
    toast.show();
    
    // Remove from DOM after hidden
    toastElement.addEventListener('hidden.bs.toast', function() {
        toastElement.remove();
    });
    
    return toast;
}

/**
 * Show success toast (shorthand)
 */
function toastSuccess(message, title = 'Success!') {
    return showToast(title, message, 'success');
}

/**
 * Show error toast (shorthand)
 */
function toastError(message, title = 'Error!') {
    return showToast(title, message, 'error');
}

/**
 * Show warning toast (shorthand)
 */
function toastWarning(message, title = 'Warning!') {
    return showToast(title, message, 'warning');
}

/**
 * Show info toast (shorthand)
 */
function toastInfo(message, title = 'Info') {
    return showToast(title, message, 'info');
}

// Laravel session flash messages integration
document.addEventListener('DOMContentLoaded', function() {
    @if(session('success'))
        toastSuccess('{{ session('success') }}');
    @endif
    
    @if(session('error'))
        toastError('{{ session('error') }}');
    @endif
    
    @if(session('warning'))
        toastWarning('{{ session('warning') }}');
    @endif
    
    @if(session('info'))
        toastInfo('{{ session('info') }}');
    @endif
    
    @if($errors->any())
        @foreach($errors->all() as $error)
            toastError('{{ $error }}', 'Validation Error');
        @endforeach
    @endif
});
</script>
@endpush
@endonce
