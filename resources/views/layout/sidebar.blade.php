<div class="sidebar-wrapper" sidebar-layout="stroke-svg">
    <div>
        <div class="logo-wrapper"><a href="/dashboard"><img class="img-fluid for-light"
                    src="{{ url('assets/images/logo/transparent.png') }}" alt=""><img class="img-fluid for-dark"
                    src="{{ url('assets/images/logo/transparent.png') }}" alt=""></a>
            <div class="back-btn"><i class="fa fa-angle-left"></i></div>
            <div class="toggle-sidebar"><i class="status_toggle middle sidebar-toggle" data-feather="grid"> </i></div>
        </div>
        <div class="logo-icon-wrapper"><a href="/dashboard"><img class="img-fluid"
                    src="{{ url('assets/images/logo/logo-icon.png') }}" alt=""></a></div>
        <nav class="sidebar-main">
            <div class="left-arrow" id="left-arrow"><i data-feather="arrow-left"></i></div>
            <div id="sidebar-menu">
                <ul class="sidebar-links" id="simple-bar">
                    <li class="back-btn"><a href="/dashboard"><img class="img-fluid"
                                src="{{ url('assets/images/logo/logo-icon.png') }}" alt=""></a>
                        <div class="mobile-back text-end"><span>Back</span><i class="fa fa-angle-right ps-2"
                                aria-hidden="true"></i>
                        </div>
                    </li>
                    <li class="sidebar-main-title">
                        <div>
                            <h6 class="lan-1">General</h6>
                        </div>
                    </li>
                    <li class="sidebar-list">
                        <a class="sidebar-link sidebar-title link-nav {{ Request::is('manage-loads') ? 'active' : '' }}"
                            href="{{ url('manage-loads') }}">
                            <svg class="stroke-icon">
                                <use href="{{ url('/assets/svg/icon-sprite.svg#delivery-box') }}"></use>
                            </svg>
                            <svg class="fill-icon">
                                <use href="{{ url('/assets/svg/icon-sprite.svg#delivery-box') }}"></use>
                            </svg>
                            <span>Manage Loads</span>
                        </a>
                    </li>

                    <li class="sidebar-list">
                        <a class="sidebar-link sidebar-title link-nav {{ Request::is('chat') ? 'active' : '' }}"
                            href="{{ url('chat') }}">
                            <svg class="stroke-icon">
                                <use href="{{ url('/assets/svg/icon-sprite.svg#stroke-chat') }}"></use>
                            </svg>
                            <svg class="fill-icon">
                                <use href="{{ url('/assets/svg/icon-sprite.svg#fill-chat') }}"></use>
                            </svg>
                            <span>Chat</span>
                        </a>
                    </li>


                </ul>
            </div>
            <div class="right-arrow" id="right-arrow"><i data-feather="arrow-right"></i></div>
        </nav>
    </div>
</div>
