# ST Loads - UX Enhancement Documentation

## Overview
This document covers the comprehensive UX improvements implemented across both user and admin portals.

---

## 🎨 Navigation & Layout Improvements

### Enhanced Sidebars

#### User Portal Sidebar
- **Organized Sections**: General, Load Management, Communication, Account
- **Visual Indicators**: Active states with gradient backgrounds
- **Icons**: Consistent Feather icons throughout
- **Badges**: Unread message counter support
- **Hover Effects**: Smooth animations and visual feedback
- **Mobile Responsive**: Collapsible  sidebar for small screens

#### Admin Portal Sidebar
- **Categorized Navigation**: Dashboard, User Management, Load Operations, System Configuration
- **Collapsible Sections**: Master Data submenu with smooth transitions
- **Enhanced Buttons**: All navigation items are full-width with consistent styling
- **Active States**: Gradient backgrounds and visual indicators
- **Responsive Design**: Fixed sidebar on mobile with overlay

### Key Features:
- Section titles with icons
- Chevron indicators on hover/active
- Smooth slide-in animations
- Professional gradient effects on active items

---

## 📍 Breadcrumb Component

### Usage

```blade
@include('components.breadcrumb', [
    'title' => 'User Management',
    'subtitle' => 'Manage all system users',  // Optional
    'items' => [
        ['label' => 'Dashboard', 'url' => route('dashboard'), 'icon' => 'home'],
        ['label' => 'Users', 'url' => route('users.index')],
        ['label' => 'Edit User']  // Current page (no URL)
    ],
    'actions' => '<button class="btn btn-primary">Add User</button>'  // Optional
])
```

### Features:
- Responsive layout (stacks on mobile)
- Optional icons on breadcrumb items
- Optional action buttons
- Automatic active state for last item
- Feather icons support

---

## ⏳ Loading Skeleton Component

### Available Types

#### 1. Table Skeleton
```blade
@include('components.loading-skeleton', [
    'type' => 'table',
    'rows' => 10  // Number of skeleton rows
])
```

#### 2. Card Skeleton
```blade
@include('components.loading-skeleton', ['type' => 'card'])
```

#### 3. List Skeleton
```blade
@include('components.loading-skeleton', [
    'type' => 'list',
    'items' => 5  // Number of list items
])
```

#### 4. Form Skeleton
```blade
@include('components.loading-skeleton', ['type' => 'form'])
```

#### 5. Dashboard Widget Skeleton
```blade
@include('components.loading-skeleton', ['type' => 'dashboard-widget'])
```

### Implementation Example

```blade
<div id="dataContainer">
    @include('components.loading-skeleton', ['type' => 'table', 'rows' => 5])
</div>

<script>
    // Fetch data
    fetch('/api/loads')
        .then(response => response.json())
        .then(data => {
            // Replace skeleton with actual data
            document.getElementById('dataContainer').innerHTML = renderTable(data);
        });
</script>
```

### Features:
- Shimmer animation effect
- Multiple pre-built layouts
- Dark mode support
- Smooth fade-in animation
- Responsive design

---

## 🔔 Toast Notification System

### JavaScript Functions

#### Basic Usage
```javascript
showToast('Title', 'Message', 'type', duration);
```

#### Shorthand Functions
```javascript
// Success toast (auto-hide: 5 seconds)
toastSuccess('User created successfully!');
toastSuccess('Changes saved', 'Done!');

// Error toast
toastError('Unable to save changes');
toastError('Invalid input', 'Validation Error');

// Warning toast
toastWarning('Please review your entries');

// Info toast
toastInfo('System update available');
```

#### Advanced Options
```javascript
// Custom duration (in milliseconds)
showToast('Processing', 'This might take a while...', 'info', 0);  // No auto-hide

// Manual close after 10 seconds
showToast('Success', 'Operation completed', 'success', 10000);
```

### Laravel Integration

The toast system automatically displays Laravel session flash messages:

```php
// In your controller
return redirect()->back()->with('success', 'User created successfully!');
return redirect()->back()->with('error', 'Something went wrong');
return redirect()->back()->with('warning', 'Please verify your email');
return redirect()->back()->with('info', 'Profile update scheduled');
```

### Validation Errors
```php
// Automatic display of validation errors
return redirect()->back()->withErrors($validator);
```

### Features:
- 4 toast types (success, error, warning, info)
- Auto-hide with custom duration
- Icon indicators with Feather icons
- Stacked toasts (multiple at once)
- Bootstrap 5 integrated
- Dismissible with close button
- Responsive positioning (top-right)

---

## ✅ Form Validation Components

### Form Input Component

```blade
@include('components.form-input', [
    'name' => 'email',
    'label' => 'Email Address',
    'type' => 'email',
    'value' => old('email', $user->email ?? ''),
    'required' => true,
    'placeholder' => 'Enter your email',
    'help' => 'We will never share your email',
    'autocomplete' => 'email',
    'class' => 'custom-class',  // Optional
    'wrapperClass' => 'mb-4'     // Optional (default: mb-3)
])
```

### Form Select Component

```blade
@include('components.form-select', [
    'name' => 'role',
    'label' => 'User Role',
    'options' => [
        2 => 'Shipper',
        3 => 'Carrier',
        4 => 'Broker',
        5 => 'Freight Forwarder'
    ],
    'selected' => old('role', $user->role_id ?? ''),
    'required' => true,
    'placeholder' => 'Select a role',
    'help' => 'Choose the user's access level'
])
```

### Features:
- Automatic Laravel validation error display
- Required field indicators (red asterisk)
- Custom help text
- Old input value preservation
- Visual error indicators
- Icons for validation states
- Consistent styling
- Full accessibility support

---

## 📱 Mobile Responsiveness

### Sidebar Behavior

#### User Portal
- **Desktop**: Fixed left sidebar
- **Tablet/Mobile**: Collapsible sidebar with toggle button
- **Toggle**: Hamburger icon in header
- **Overlay**: Dark overlay when sidebar is open on mobile

#### Admin Portal
- **Desktop**: Fixed left column (3-column grid)
- **Tablet/Mobile**: Hidden sidebar, shows on toggle
- **Positioning**: Fixed positioning with smooth slide-in
- **Z-index**: Above content but below modals

### Breakpoints
- Desktop: > 1199px
- Tablet: 768px - 1199px  
- Mobile: < 768px

### Features:
- Touch-friendly tap targets (minimum 44x44px)
- Swipe gestures (future enhancement)
- Responsive navigation buttons
- Adaptive font sizes
- Flexible grid layouts

---

## 🎯 Real-Time Features

### Unread Message Counter

```javascript
// Update message counter in sidebar
function updateMessageCount(count) {
    const badge = document.getElementById('unread-messages-count');
    if (count > 0) {
        badge.textContent = count;
        badge.classList.remove('d-none');
    } else {
        badge.classList.add('d-none');
    }
}

// Example with Laravel Echo/Pusher
Echo.private(`user.${userId}`)
    .listen('MessageSent', (e) => {
        updateMessageCount(e.unreadCount);
        toastInfo(`New message from ${e.sender}`);
    });
```

---

## 🎨 Styling Best Practices

### Colors
- Primary: `#7366ff` (Purple)
- Success: `#28a745` (Green)
- Error: `#dc3545` (Red)
- Warning: `#ffc107` (Yellow)
- Info: `#17a2b8` (Cyan)

### Typography
- Font Family: Rubik, Roboto
- Headings: 600 weight
- Body: 400 weight
- Small Text: 0.75rem - 0.875rem

### Spacing
- Section margins: 1rem - 2rem
- Card padding: 1.25rem
- Button padding: 0.375rem 0.75rem
- Form field margin-bottom: 1rem (mb-3)

### Shadows
- Light: `0 2px 8px rgba(0,0,0,0.08)`
- Medium: `0 4px 12px rgba(0,0,0,0.15)`
- Heavy: `0 8px 24px rgba(0,0,0,0.20)`

---

## 🔧 Quick Integration Examples

### Complete Page with All Components

```blade
@extends('layout.app')

@section('content')
    <!-- Breadcrumb -->
    @include('components.breadcrumb', [
        'title' => 'Manage Loads',
        'items' => [
            ['label' => 'Dashboard', 'url' => route('dashboard'), 'icon' => 'home'],
            ['label' => 'Loads', 'url' => route('loads.index')],
            ['label' => 'All Loads']
        ],
        'actions' => '<a href="' . route('loads.add') . '" class="btn btn-primary">
                         <i data-feather="plus"></i> Create Load
                      </a>'
    ])

    <div class="container-fluid">
        <div class="card">
            <div class="card-body">
                <!-- Loading state -->
                <div id="loadingState">
                    @include('components.loading-skeleton', ['type' => 'table', 'rows' => 10])
                </div>

                <!-- Actual content (hidden initially) -->
                <div id="dataContent" style="display: none;">
                    <table class="table">
                        <!-- Your table content -->
                    </table>
                </div>
            </div>
        </div>
    </div>
@endsection

@push('scripts')
<script>
    document.addEventListener('DOMContentLoaded', function() {
        // Simulate data loading
        setTimeout(() => {
            document.getElementById('loadingState').style.display = 'none';
            document.getElementById('dataContent').style.display = 'block';
            feather.replace();
            toastSuccess('Loads loaded successfully!');
        }, 1000);
    });
</script>
@endpush
```

### Form with Validation

```blade
<form action="{{ route('users.store') }}" method="POST">
    @csrf

    @include('components.form-input', [
        'name' => 'name',
        'label' => 'Full Name',
        'required' => true,
        'placeholder' => 'John Doe'
    ])

    @include('components.form-input', [
        'name' => 'email',
        'label' => 'Email',
        'type' => 'email',
        'required' => true,
        'autocomplete' => 'email'
    ])

    @include('components.form-select', [
        'name' => 'role',
        'label' => 'Role',
        'options' => $roles,
        'required' => true
    ])

    <button type="submit" class="btn btn-primary">Create User</button>
</form>
```

---

## 📊 Performance Considerations

### Best Practices

1. **Lazy Loading**: Use skeletons while fetching data
2. **Toast Limits**: Auto-dismiss after 5 seconds to avoid clutter
3. **Icon Loading**: Feather icons loaded once, replaced globally
4. **CSS Animations**: Hardware-accelerated transforms
5. **Component Reuse**: Include components, don't duplicate code

### Optimization Tips

```javascript
// Debounce search inputs
let searchTimeout;
document.getElementById('search').addEventListener('input', function(e) {
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => {
        performSearch(e.target.value);
    }, 300);
});

// Batch toast notifications
let toastQueue = [];
function queueToast(title, message, type) {
    toastQueue.push({title, message, type});
    if (toastQueue.length === 1) {
        processToastQueue();
    }
}

function processToastQueue() {
    if (toastQueue.length === 0) return;
    const {title, message, type} = toastQueue.shift();
    showToast(title, message, type);
    setTimeout(processToastQueue, 500);
}
```

---

## 🐛 Troubleshooting

### Icons Not Showing
```javascript
// Ensure feather.replace() is called after dynamic content
feather.replace();
```

### Toasts Not Appearing
```blade
<!-- Make sure toast container is included in layout -->
@include('components.toast-container')
```

### Breadcrumbs Overlap on Mobile
```css
/* Custom CSS if needed */
@media (max-width: 767px) {
    .page-title .row {
        flex-direction: column;
    }
}
```

### Skeleton Not Loading
```blade
<!-- Check @once directive isn't preventing styles -->
<!-- Styles should load only once per page, not per instance -->
```

---

## 📝 Change Log

### Version 1.0 (March 2026)
- ✅ Enhanced user sidebar with sections
- ✅ Enhanced admin sidebar with categorization
- ✅ Breadcrumb component system
- ✅ Loading skeleton components (5 types)
- ✅ Toast notification system
- ✅ Form validation components (input, select)
- ✅ Mobile responsive navigation
- ✅ Improved visual feedback and animations
- ✅ Laravel session flash integration
- ✅ Dark mode support for skeletons

---

## 🚀 Future Enhancements

### Planned Features
- [ ] Notification center dropdown
- [ ] Advanced search with filters
- [ ] Bulk action toasts
- [ ] Real-time load status updates
- [ ] Push notifications
- [ ] Keyboard shortcuts
- [ ] Drag-and-drop file uploads with progress
- [ ] Advanced data tables with sorting/filtering
- [ ] Charts and analytics dashboard

---

## 🌓 Dark Mode System

### Overview
ST Loads now features a comprehensive dark mode system that preserves the blue background theme while converting all UI elements to a dark color scheme.

### Features
- **Persistent State**: Remembers user preference via localStorage
- **Toggle Button**: Accessible in top-right corner of both user and admin portals
- **Smooth Transitions**: 0.3s ease transitions between modes
- **Icon Indicators**: Moon icon for light mode, Sun icon for dark mode
- **Toast Notifications**: Optional notifications on theme change
- **Full Coverage**: Affects sidebar, cards, tables, forms, modals, and more

### Usage
Click the moon/sun icon in the top-right corner of the header to toggle dark mode. Your preference is automatically saved and will persist across sessions.

### What Changes in Dark Mode
✅ **Sidebar**: Dark gradient background (#1a1a2e → #16213e)  
✅ **Cards**: Dark gradient (#1f2937 → #1a202c)  
✅ **Tables**: Dark backgrounds with subtle borders  
✅ **Forms**: Dark inputs with light text  
✅ **Modals**: Dark backgrounds matching card theme  
✅ **Dropdowns**: Consistent dark styling  
✅ **Breadcrumbs**: Dark with light text  

❌ **Background**: Stays blue theme (as requested)  
❌ **Badges**: Keep original colors for clarity  
❌ **Primary Buttons**: Maintain purple gradient  

### Color Palette
- **Background Gradient**: `#1a1a2e` → `#16213e` (sidebar), `#1f2937` → `#1a202c` (cards)
- **Text**: `#e0e0e0` (primary), `#fff` (headings)
- **Borders**: `rgba(255, 255, 255, 0.1)`
- **Hover**: `rgba(115, 102, 255, 0.2)`
- **Active**: Purple gradient maintained (`#7366ff` → `#5e54d9`)

### Technical Details
```html
<!-- Toggle Button (automatically included in header) -->
<button class="mode-toggle-btn" id="darkModeToggle">
    <i data-feather="moon" class="moon-icon"></i>
    <i data-feather="sun" class="sun-icon d-none"></i>
</button>
```

```javascript
// Check current state
const isDarkMode = document.body.classList.contains('dark-mode');

// Manually toggle (if needed in custom scripts)
document.body.classList.toggle('dark-mode');
localStorage.setItem('darkMode', 'enabled'); // or 'disabled'
```

```css
/* Custom dark mode styles for your components */
body.dark-mode .your-component {
    background: rgba(255, 255, 255, 0.05);
    color: #e0e0e0;
}
```

### Browser Support
- Chrome/Edge: Full support
- Firefox: Full support
- Safari: Full support
- IE11: Graceful degradation (no localStorage persistence)

---

## 📞 Support

For questions about these UX enhancements, refer to:
- Component files in `resources/views/components/`
- Sidebar implementations in `resources/views/layout/sidebar.blade.php` and `resources/views/admin-layout/sidebar.blade.php`
- This documentation file

---

**Last Updated**: March 31, 2026
**Version**: 1.0
**Author**: Development Team
