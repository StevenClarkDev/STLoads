<div class="col-xl-3 box-col-6 ps-3 pt-2">
    <div class="md-sidebar">
        <div class="md-sidebar-aside job-left-aside custom-scrollbar">
            <div class="file-sidebar">
                <div class="card admin-sidebar-card">
                    <div class="card-body">
                        @php
                            $currentRoute = Route::currentRouteName();
                            $currentRoleId = request()->route('id');
                            use Illuminate\Support\Str;

                            $isMasterActive =
                                Str::startsWith($currentRoute, 'load_types') ||
                                Str::startsWith($currentRoute, 'equipments') ||
                                Str::startsWith($currentRoute, 'commodity_types') ||
                                Str::startsWith($currentRoute, 'locations');
                        @endphp

                        <!-- Main Navigation Section -->
                        <div class="admin-nav-section mb-4">
                            <h6 class="admin-nav-title">
                                <i data-feather="grid" class="me-2"></i>
                                Dashboard
                            </h6>
                            <ul class="nav flex-column">
                                <li class="nav-item mb-2">
                                    <a href="{{ route('admin_dashboard') }}" class="admin-nav-link">
                                        <div class="btn btn-{{ request()->routeIs('admin_dashboard') ? 'primary' : 'light' }} w-100 text-start">
                                            <i data-feather="home" class="me-2"></i> Overview
                                            <i data-feather="chevron-right" class="float-end mt-1"></i>
                                        </div>
                                    </a>
                                </li>
                            </ul>
                        </div>

                        <!-- User Management Section -->
                        <div class="admin-nav-section mb-4">
                            <h6 class="admin-nav-title">
                                <i data-feather="users" class="me-2"></i>
                                User Management
                            </h6>
                            <ul class="nav flex-column">
                                <li class="nav-item mb-2">
                                    <a href="{{ route('user_approval') }}" class="admin-nav-link">
                                        <div class="btn btn-{{ request()->routeIs('user_approval') ? 'primary' : 'light' }} w-100 text-start">
                                            <i data-feather="user-check" class="me-2"></i> Pending Approvals
                                            <i data-feather="chevron-right" class="float-end mt-1"></i>
                                        </div>
                                    </a>
                                </li>
                                <li class="nav-item mb-2">
                                    <a href="{{ route('users.index') }}" class="admin-nav-link">
                                        <div class="btn btn-{{ Str::startsWith($currentRoute, 'users.') ? 'primary' : 'light' }} w-100 text-start">
                                            <i data-feather="users" class="me-2"></i> All Users
                                            <i data-feather="chevron-right" class="float-end mt-1"></i>
                                        </div>
                                    </a>
                                </li>
                                <li class="nav-item mb-2">
                                    <a href="{{ route('users_by_role', 3) }}" class="admin-nav-link">
                                        <div class="btn btn-{{ $currentRoute === 'users_by_role' && $currentRoleId == 3 ? 'primary' : 'light' }} w-100 text-start">
                                            <i data-feather="truck" class="me-2"></i> Carriers
                                            <i data-feather="chevron-right" class="float-end mt-1"></i>
                                        </div>
                                    </a>
                                </li>
                                <li class="nav-item mb-2">
                                    <a href="{{ route('users_by_role', 2) }}" class="admin-nav-link">
                                        <div class="btn btn-{{ $currentRoute === 'users_by_role' && $currentRoleId == 2 ? 'primary' : 'light' }} w-100 text-start">
                                            <i data-feather="box" class="me-2"></i> Shippers
                                            <i data-feather="chevron-right" class="float-end mt-1"></i>
                                        </div>
                                    </a>
                                </li>
                                <li class="nav-item mb-2">
                                    <a href="{{ route('users_by_role', 4) }}" class="admin-nav-link">
                                        <div class="btn btn-{{ $currentRoute === 'users_by_role' && $currentRoleId == 4 ? 'primary' : 'light' }} w-100 text-start">
                                            <i data-feather="briefcase" class="me-2"></i> Brokers
                                            <i data-feather="chevron-right" class="float-end mt-1"></i>
                                        </div>
                                    </a>
                                </li>
                                <li class="nav-item mb-2">
                                    <a href="{{ route('users_by_role', 5) }}" class="admin-nav-link">
                                        <div class="btn btn-{{ $currentRoute === 'users_by_role' && $currentRoleId == 5 ? 'primary' : 'light' }} w-100 text-start">
                                            <i data-feather="send" class="me-2"></i> Freight Forwarders
                                            <i data-feather="chevron-right" class="float-end mt-1"></i>
                                        </div>
                                    </a>
                                </li>
                                <li class="nav-item mb-2">
                                    <a href="{{ url('roles') }}" class="admin-nav-link">
                                        <div class="btn btn-{{ Str::startsWith($currentRoute, 'roles.') ? 'primary' : 'light' }} w-100 text-start">
                                            <i data-feather="shield" class="me-2"></i> Roles & Permissions
                                            <i data-feather="chevron-right" class="float-end mt-1"></i>
                                        </div>
                                    </a>
                                </li>
                            </ul>
                        </div>

                        <!-- Load Management Section -->
                        <div class="admin-nav-section mb-4">
                            <h6 class="admin-nav-title">
                                <i data-feather="package" class="me-2"></i>
                                Load Operations
                            </h6>
                            <ul class="nav flex-column">
                                <li class="nav-item mb-2">
                                    <a href="{{ url('admin-manage-loads') }}" class="admin-nav-link">
                                        <div class="btn btn-{{ Str::startsWith($currentRoute, 'admin-manage-loads.') ? 'primary' : 'light' }} w-100 text-start">
                                            <i data-feather="truck" class="me-2"></i> Manage Loads
                                            <i data-feather="chevron-right" class="float-end mt-1"></i>
                                        </div>
                                    </a>
                                </li>
                                <li class="nav-item mb-2">
                                    <a href="{{ route('stloads.operations') }}" class="admin-nav-link">
                                        <div class="btn btn-{{ request()->routeIs('stloads.operations') || request()->routeIs('stloads.handoff.show') ? 'primary' : 'light' }} w-100 text-start">
                                            <i data-feather="radio" class="me-2"></i> STLOADS Operations
                                            <i data-feather="chevron-right" class="float-end mt-1"></i>
                                        </div>
                                    </a>
                                </li>
                                <li class="nav-item mb-2">
                                    <a href="{{ route('stloads.reconciliation') }}" class="admin-nav-link">
                                        <div class="btn btn-{{ request()->routeIs('stloads.reconciliation') ? 'primary' : 'light' }} w-100 text-start">
                                            <i data-feather="git-pull-request" class="me-2"></i> Reconciliation
                                            <i data-feather="chevron-right" class="float-end mt-1"></i>
                                        </div>
                                    </a>
                                </li>
                            </ul>
                        </div>

                        <!-- System Configuration Section -->
                        <div class="admin-nav-section mb-4">
                            <h6 class="admin-nav-title">
                                <i data-feather="settings" class="me-2"></i>
                                System Configuration
                            </h6>
                            <ul class="nav flex-column">
                                <li class="nav-item mb-2">
                                    <button
                                        class="btn btn-{{ $isMasterActive ? 'primary' : 'light' }} w-100 text-start d-flex align-items-center justify-content-between"
                                        type="button" data-bs-toggle="collapse" data-bs-target="#masterMenu"
                                        aria-expanded="{{ $isMasterActive ? 'true' : 'false' }}">
                                        <span>
                                            <i data-feather="database" class="me-2"></i> Master Data
                                        </span>
                                        <i data-feather="{{ $isMasterActive ? 'chevron-down' : 'chevron-right' }}" class="collapse-icon"></i>
                                    </button>
                                    <div class="collapse {{ $isMasterActive ? 'show' : '' }} mt-2" id="masterMenu">
                                        <ul class="nav flex-column ms-3">
                                            <li class="nav-item mb-2">
                                                <a href="{{ route('load_types.index') }}" class="w-100">
                                                    <div class="btn btn-{{ Str::startsWith($currentRoute, 'load_types') ? 'primary' : 'light' }} btn-sm w-100 text-start">
                                                        <i data-feather="layers" class="me-2"></i> Load Types
                                                    </div>
                                                </a>
                                            </li>
                                            <li class="nav-item mb-2">
                                                <a href="{{ route('equipments.index') }}" class="w-100">
                                                    <div class="btn btn-{{ Str::startsWith($currentRoute, 'equipments') ? 'primary' : 'light' }} btn-sm w-100 text-start">
                                                        <i data-feather="tool" class="me-2"></i> Equipments
                                                    </div>
                                                </a>
                                            </li>
                                            <li class="nav-item mb-2">
                                                <a href="{{ route('commodity_types.index') }}" class="w-100">
                                                    <div class="btn btn-{{ Str::startsWith($currentRoute, 'commodity_types') ? 'primary' : 'light' }} btn-sm w-100 text-start">
                                                        <i data-feather="package" class="me-2"></i> Commodity Types
                                                    </div>
                                                </a>
                                            </li>
                                            <li class="nav-item mb-2">
                                                <a href="{{ route('locations.index') }}" class="w-100">
                                                    <div class="btn btn-{{ Str::startsWith($currentRoute, 'locations') ? 'primary' : 'light' }} btn-sm w-100 text-start">
                                                        <i data-feather="map-pin" class="me-2"></i> Locations
                                                    </div>
                                                </a>
                                            </li>
                                        </ul>
                                    </div>
                                </li>
                            </ul>
                        </div>

                        <hr>

                        <ul class="nav flex-column">
                            <li class="nav-item">
                                <form method="POST" action="{{ route('logout') }}">
                                    @csrf
                                    <button type="submit"
                                        class="btn btn-outline-primary w-100 d-flex align-items-center justify-content-center">
                                        <i data-feather="log-out"></i> Logout
                                    </button>
                                </form>
                            </li>
                        </ul>

                    </div>
                </div>
            </div>
        </div>
    </div>
</div>

<link rel="stylesheet" type="text/css" href="{{ url('assets/css/vendors/feather-icon.css') }}">
<script src="https://unpkg.com/feather-icons"></script>
<script>
    feather.replace();
</script>

@push('styles')
<style>
    /* Enhanced Admin Sidebar Styles */
    .admin-sidebar-card {
        border: none;
        box-shadow: 0 0 20px rgba(0,0,0,0.08);
        border-radius: 12px;
    }
    
    .admin-nav-section {
        border-bottom: 1px solid #f0f0f0;
        padding-bottom: 1rem;
    }
    
    .admin-nav-section:last-of-type {
        border-bottom: none;
    }
    
    .admin-nav-title {
        font-size: 0.75rem;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        color: #7366ff;
        margin-bottom: 0.75rem;
        display: flex;
        align-items: center;
    }
    
    .admin-nav-title i {
        width: 16px;
        height: 16px;
    }
    
    .admin-nav-link {
        text-decoration: none;
        display: block;
    }
    
    .admin-nav-link .btn {
        transition: all 0.3s ease;
        position: relative;
        overflow: hidden;
    }
    
    .admin-nav-link .btn::before {
        content: '';
        position: absolute;
        top: 50%;
        left: 0;
        width: 4px;
        height: 0;
        background: #7366ff;
        transition: all 0.3s ease;
        transform: translateY(-50%);
        border-radius: 0 4px 4px 0;
    }
    
    .admin-nav-link .btn:hover::before,
    .admin-nav-link .btn-primary::before {
        height: 60%;
    }
    
    .admin-nav-link .btn:hover {
        transform: translateX(4px);
        box-shadow: 0 2px 8px rgba(0,0,0,0.08);
    }
    
    .admin-nav-link .btn-primary {
        background: linear-gradient(135deg, #7366ff 0%, #5e54d9 100%);
        border: none;
        box-shadow: 0 4px 12px rgba(115, 102, 255, 0.3);
    }
    
    .admin-nav-link .btn i.float-end {
        font-size: 14px;
        opacity: 0;
        transition: opacity 0.3s ease;
    }
    
    .admin-nav-link:hover .btn i.float-end,
    .admin-nav-link .btn-primary i.float-end {
        opacity: 1;
    }
    
    .collapse-icon {
        transition: transform 0.3s ease;
        width: 18px;
        height: 18px;
    }
    
    .btn[aria-expanded="true"] .collapse-icon {
        transform: rotate(90deg);
    }
    
    /* Master menu submenu */
    #masterMenu .btn-sm {
        font-size: 0.875rem;
        padding: 0.5rem 0.75rem;
    }
    
    #masterMenu {
        background: rgba(115, 102, 255, 0.03);
        border-left: 2px solid #7366ff;
        padding: 0.5rem;
        border-radius: 8px;
        margin-left: 0.5rem;
    }
    
    /* Logout button */
    .btn-outline-primary:hover {
        background: #7366ff;
        color: white;
    }
    
    /* Mobile Responsive */
    @media (max-width: 1199px) {
        .md-sidebar {
            position: fixed;
            left: -300px;
            top: 0;
            height: 100vh;
            width: 300px;
            z-index: 1050;
            background: white;
            transition: left 0.3s ease;
            overflow-y: auto;
        }
        
        .md-sidebar.show {
            left: 0;
            box-shadow: 2px 0 20px rgba(0,0,0,0.2);
        }
        
        .admin-nav-link .btn:hover {
            transform: none;
        }
    }
    
    /* Smooth animations */
    .admin-nav-section {
        animation: fadeInUp 0.4s ease forwards;
    }
    
    .admin-nav-section:nth-child(1) { animation-delay: 0.1s; }
    .admin-nav-section:nth-child(2) { animation-delay: 0.2s; }
    .admin-nav-section:nth-child(3) { animation-delay: 0.3s; }
    .admin-nav-section:nth-child(4) { animation-delay: 0.4s; }
    
    @keyframes fadeInUp {
        from {
            opacity: 0;
            transform: translateY(10px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }
</style>
@endpush