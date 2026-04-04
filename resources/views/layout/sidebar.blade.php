<div class="sidebar-wrapper" sidebar-layout="stroke-svg">
    <div>
        <!-- Logo Section -->
        <div class="logo-wrapper">
            <a href="{{ route('dashboard') }}">
                <img class="img-fluid for-light" src="{{ url('assets/images/logo/transparent.png') }}" alt="">
                <img class="img-fluid for-dark" src="{{ url('assets/images/logo/transparent.png') }}" alt="">
            </a>
            <div class="back-btn"><i class="fa fa-angle-left"></i></div>
            <div class="toggle-sidebar">
                <i class="status_toggle middle sidebar-toggle" data-feather="grid"></i>
            </div>
        </div>

        <div class="logo-icon-wrapper">
            <a href="{{ route('dashboard') }}">
                <img class="img-fluid" src="{{ url('assets/images/logo/logo-icon.png') }}" alt="">
            </a>
        </div>

        <!-- Sidebar Menu -->
        <nav class="sidebar-main">
            <div class="left-arrow" id="left-arrow"><i data-feather="arrow-left"></i></div>
            <div id="sidebar-menu">
                <ul class="sidebar-links" id="simple-bar">

                    <!-- Back Button -->
                    <li class="back-btn">
                        <a href="{{ route('dashboard') }}">
                            <img class="img-fluid" src="{{ url('assets/images/logo/logo-icon.png') }}" alt="">
                        </a>
                        <div class="mobile-back text-end">
                            <span>Back</span>
                            <i class="fa fa-angle-right ps-2" aria-hidden="true"></i>
                        </div>
                    </li>

                    <!-- General Section -->
                    <li class="sidebar-main-title">
                        <div>
                            <h6 class="lan-1">
                                <i data-feather="grid" class="section-icon me-2"></i>
                                General
                            </h6>
                        </div>
                    </li>

                    <!-- Dashboard -->
                    <li class="sidebar-list">
                        <a class="sidebar-link sidebar-title link-nav {{ Request::is('dashboard') ? 'active' : '' }}"
                            href="{{ route('dashboard') }}">
                            <i data-feather="home" class="me-2"></i>
                            <span>Dashboard</span>
                            <div class="sidebar-badge">
                                <i data-feather="chevron-right" class="nav-arrow"></i>
                            </div>
                        </a>
                    </li>

                    <!-- Operations Section -->
                    <li class="sidebar-main-title">
                        <div>
                            <h6 class="lan-1">
                                <i data-feather="package" class="section-icon me-2"></i>
                                Load Management
                            </h6>
                        </div>
                    </li>

                    <!-- Manage Loads -->
                    <li class="sidebar-list">
                        <a class="sidebar-link sidebar-title link-nav {{ Request::is('manage-loads') ? 'active' : '' }}"
                            href="{{ url('manage-loads') }}">
                            <i data-feather="truck" class="me-2"></i>
                            <span>My Loads</span>
                            <div class="sidebar-badge">
                                <i data-feather="chevron-right" class="nav-arrow"></i>
                            </div>
                        </a>
                    </li>

                    <!-- Dispatch Desks Section -->
                    <li class="sidebar-main-title">
                        <div>
                            <h6 class="lan-1">
                                <i data-feather="layout" class="section-icon me-2"></i>
                                Dispatch Desks
                            </h6>
                        </div>
                    </li>
                    <li class="sidebar-list">
                        <a class="sidebar-link sidebar-title link-nav {{ Request::is('desk/quote') ? 'active' : '' }}"
                            href="{{ route('desk.quote') }}">
                            <i data-feather="dollar-sign" class="me-2"></i>
                            <span>Quote Desk</span>
                            <div class="sidebar-badge"><i data-feather="chevron-right" class="nav-arrow"></i></div>
                        </a>
                    </li>
                    <li class="sidebar-list">
                        <a class="sidebar-link sidebar-title link-nav {{ Request::is('desk/tender') ? 'active' : '' }}"
                            href="{{ route('desk.tender') }}">
                            <i data-feather="send" class="me-2"></i>
                            <span>Tender Desk</span>
                            <div class="sidebar-badge"><i data-feather="chevron-right" class="nav-arrow"></i></div>
                        </a>
                    </li>
                    <li class="sidebar-list">
                        <a class="sidebar-link sidebar-title link-nav {{ Request::is('desk/facility') ? 'active' : '' }}"
                            href="{{ route('desk.facility') }}">
                            <i data-feather="map-pin" class="me-2"></i>
                            <span>Facility Desk</span>
                            <div class="sidebar-badge"><i data-feather="chevron-right" class="nav-arrow"></i></div>
                        </a>
                    </li>
                    <li class="sidebar-list">
                        <a class="sidebar-link sidebar-title link-nav {{ Request::is('desk/closeout') ? 'active' : '' }}"
                            href="{{ route('desk.closeout') }}">
                            <i data-feather="check-square" class="me-2"></i>
                            <span>Closeout Desk</span>
                            <div class="sidebar-badge"><i data-feather="chevron-right" class="nav-arrow"></i></div>
                        </a>
                    </li>
                    <li class="sidebar-list">
                        <a class="sidebar-link sidebar-title link-nav {{ Request::is('desk/collections') ? 'active' : '' }}"
                            href="{{ route('desk.collections') }}">
                            <i data-feather="credit-card" class="me-2"></i>
                            <span>Collections Desk</span>
                            <div class="sidebar-badge"><i data-feather="chevron-right" class="nav-arrow"></i></div>
                        </a>
                    </li>

                    <!-- STLOADS Integration Section -->
                    <li class="sidebar-main-title">
                        <div>
                            <h6 class="lan-1">
                                <i data-feather="radio" class="section-icon me-2"></i>
                                STLOADS Integration
                            </h6>
                        </div>
                    </li>

                    <!-- STLOADS Operations -->
                    <li class="sidebar-list">
                        <a class="sidebar-link sidebar-title link-nav {{ Request::is('stloads/operations*') ? 'active' : '' }}"
                            href="{{ route('stloads.operations') }}">
                            <i data-feather="radio" class="me-2"></i>
                            <span>STLOADS Ops</span>
                            <div class="sidebar-badge">
                                <i data-feather="chevron-right" class="nav-arrow"></i>
                            </div>
                        </a>
                    </li>
                    <li class="sidebar-list">
                        <a class="sidebar-link sidebar-title link-nav {{ Request::is('stloads/reconciliation*') ? 'active' : '' }}"
                            href="{{ route('stloads.reconciliation') }}">
                            <i data-feather="git-pull-request" class="me-2"></i>
                            <span>Reconciliation</span>
                            <div class="sidebar-badge">
                                <i data-feather="chevron-right" class="nav-arrow"></i>
                            </div>
                        </a>
                    </li>

                    <!-- Communication Section -->
                    <li class="sidebar-main-title">
                        <div>
                            <h6 class="lan-1">
                                <i data-feather="message-circle" class="section-icon me-2"></i>
                                Communication
                            </h6>
                        </div>
                    </li>

                    <!-- Chat -->
                    <li class="sidebar-list">
                        <a class="sidebar-link sidebar-title link-nav {{ Request::is('chat') ? 'active' : '' }}"
                            href="{{ route('chat.index') }}">
                            <i data-feather="message-square" class="me-2"></i>
                            <span>Messages</span>
                            <div class="sidebar-badge">
                                <span class="badge badge-sm bg-danger d-none" id="unread-messages-count">0</span>
                                <i data-feather="chevron-right" class="nav-arrow"></i>
                            </div>
                        </a>
                    </li>

                    <!-- Account Section -->
                    <li class="sidebar-main-title">
                        <div>
                            <h6 class="lan-1">
                                <i data-feather="user" class="section-icon me-2"></i>
                                Account
                            </h6>
                        </div>
                    </li>

                    <!-- Profile -->
                    <li class="sidebar-list">
                        <a class="sidebar-link sidebar-title link-nav {{ Request::is('profile/' . Auth::user()->id) ? 'active' : '' }}"
                            href="{{ route('profile', Auth::user()->id) }}">
                            <i data-feather="user-check" class="me-2"></i>
                            <span>My Profile</span>
                            <div class="sidebar-badge">
                                <i data-feather="chevron-right" class="nav-arrow"></i>
                            </div>
                        </a>
                    </li>
                </ul>
            </div>
            <div class="right-arrow" id="right-arrow"><i data-feather="arrow-right"></i></div>
        </nav>
    </div>
</div>

<!-- Initialize Feather Icons -->
@section('scripts')
    <script>
        feather.replace();
    </script>
@endsection

@push('styles')
<style>
    /* Enhanced Sidebar Styles */
    .sidebar-main-title h6 {
        display: flex;
        align-items: center;
        font-size: 0.75rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        color: #7366ff;
        margin-top: 1.5rem;
        margin-bottom: 0.5rem;
    }
    
    .section-icon {
        width: 16px;
        height: 16px;
        opacity: 0.7;
    }
    
    .sidebar-list .sidebar-link {
        position: relative;
        display: flex;
        align-items: center;
        padding: 12px 20px;
        border-radius: 8px;
        transition: all 0.3s ease;
        margin-bottom: 4px;
    }
    
    .sidebar-list .sidebar-link:hover {
        background: rgba(115, 102, 255, 0.1);
        transform: translateX(5px);
    }
    
    .sidebar-list .sidebar-link.active {
        background: linear-gradient(135deg, #7366ff 0%, #5e54d9 100%);
        color: white;
        box-shadow: 0 4px 12px rgba(115, 102, 255, 0.3);
    }
    
    .sidebar-list .sidebar-link.active i {
        color: white;
    }
    
    .sidebar-badge {
        margin-left: auto;
        display: flex;
        align-items: center;
        gap: 8px;
    }
    
    .nav-arrow {
        width: 14px;
        height: 14px;
        opacity: 0;
        transition: all 0.3s ease;
    }
    
    .sidebar-list .sidebar-link:hover .nav-arrow,
    .sidebar-list .sidebar-link.active .nav-arrow {
        opacity: 1;
    }
    
    .badge-sm {
        font-size: 0.65rem;
        padding: 0.25em 0.5em;
        min-width: 20px;
        height: 20px;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        border-radius: 10px;
    }
    
    /* Mobile Responsive */
    @media (max-width: 991px) {
        .sidebar-wrapper {
            position: fixed;
            left: -280px;
            top: 0;
            height: 100vh;
            width: 280px;
            z-index: 1050;
            background: white;
            transition: left 0.3s ease;
            box-shadow: 2px 0 20px rgba(0,0,0,0.1);
        }
        
        .sidebar-wrapper.show {
            left: 0;
        }
        
        .sidebar-list .sidebar-link:hover {
            transform: none;
        }
    }
    
    /* Sidebar Animations */
    @keyframes slideInLeft {
        from {
            opacity: 0;
            transform: translateX(-20px);
        }
        to {
            opacity: 1;
            transform: translateX(0);
        }
    }
    
    .sidebar-list {
        animation: slideInLeft 0.3s ease forwards;
    }
    
    .sidebar-list:nth-child(1) { animation-delay: 0.05s; }
    .sidebar-list:nth-child(2) { animation-delay: 0.1s; }
    .sidebar-list:nth-child(3) { animation-delay: 0.15s; }
    .sidebar-list:nth-child(4) { animation-delay: 0.2s; }
    .sidebar-list:nth-child(5) { animation-delay: 0.25s; }
    .sidebar-list:nth-child(6) { animation-delay: 0.3s; }
</style>
@endpush