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
                            <h6 class="lan-1">General</h6>
                        </div>
                    </li>

                    <!-- Dashboard -->
                    <li class="sidebar-list">
                        <a class="sidebar-link sidebar-title link-nav {{ Request::is('dashboard') ? 'active' : '' }}"
                            href="{{ route('dashboard') }}">
                            <i data-feather="home" class="me-2"></i>
                            <span>Dashboard</span>
                        </a>
                    </li>

                    <!-- Operations Section -->
                    <li class="sidebar-main-title">
                        <div>
                            <h6 class="lan-1">Operations</h6>
                        </div>
                    </li>

                    <!-- Manage Loads -->
                    <li class="sidebar-list">
                        <a class="sidebar-link sidebar-title link-nav {{ Request::is('manage-loads') ? 'active' : '' }}"
                            href="{{ url('manage-loads') }}">
                            <i data-feather="package" class="me-2"></i>
                            <span>Manage Loads</span>
                        </a>
                    </li>

                    <!-- Chat -->
                    <li class="sidebar-list">
                        <a class="sidebar-link sidebar-title link-nav {{ Request::is('chat') ? 'active' : '' }}"
                            href="{{ route('chat.index') }}">
                            <i data-feather="message-square" class="me-2"></i>
                            <span>Chat</span>
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