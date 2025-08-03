<div class="col-xl-3 box-col-6 ps-3 pt-2">
    <div class="md-sidebar">
        <div class="md-sidebar-aside job-left-aside custom-scrollbar">
            <div class="file-sidebar">
                <div class="card">
                    <div class="card-body">
                        @php
                            $currentRoute = Route::currentRouteName();
                            $currentRoleId = request()->route('id');
                        @endphp

                        <ul class="nav flex-column">
                            <li class="nav-item mb-2">
                                <a href="{{ route('user_approval') }}">
                                    <div
                                        class="btn btn-{{ request()->routeIs('user_approval') ? 'primary' : 'light' }}">
                                        <i data-feather="home"></i> Home
                                    </div>
                                </a>
                            </li>

                            <li class="nav-item mb-2">
                                <a href="{{ route('users_by_role', 3) }}">
                                    <div
                                        class="btn btn-{{ $currentRoute === 'users_by_role' && $currentRoleId == 3 ? 'primary' : 'light' }}">
                                        <i data-feather="truck"></i> Carriers
                                    </div>
                                </a>
                            </li>

                            <li class="nav-item mb-2">
                                <a href="{{ route('users_by_role', 2) }}">
                                    <div
                                        class="btn btn-{{ $currentRoute === 'users_by_role' && $currentRoleId == 2 ? 'primary' : 'light' }}">
                                        <i data-feather="box"></i> Shippers
                                    </div>
                                </a>
                            </li>

                            <li class="nav-item mb-2">
                                <a href="{{ route('users_by_role', 4) }}">
                                    <div
                                        class="btn btn-{{ $currentRoute === 'users_by_role' && $currentRoleId == 4 ? 'primary' : 'light' }}">
                                        <i data-feather="user-check"></i> Brookers
                                    </div>
                                </a>
                            </li>

                            <li class="nav-item mb-2">
                                <a href="{{ route('users_by_role', 5) }}">
                                    <div
                                        class="btn btn-{{ $currentRoute === 'users_by_role' && $currentRoleId == 5 ? 'primary' : 'light' }}">
                                        <i data-feather="send"></i> Freight Forwarders
                                    </div>
                                </a>
                            </li>

                            <li class="nav-item mb-2">
                                <a href="{{ url('roles') }}">
                                    <div
                                        class="btn btn-{{ Str::startsWith($currentRoute, 'roles.') ? 'primary' : 'light' }}">
                                        <i data-feather="shield"></i> Roles
                                    </div>
                                </a>
                            </li>
                            <li class="nav-item mb-2">
                                <a href="{{ url('manage-loads') }}">
                                    <div
                                        class="btn btn-{{ Str::startsWith($currentRoute, 'manage-loads.') ? 'primary' : 'light' }}">
                                        <i data-feather="truck"></i> Manage Loads
                                    </div>
                                </a>
                            </li>
                            <li class="nav-item mb-2">
                                <a href="{{ url('chat') }}">
                                    <div
                                        class="btn btn-{{ Str::startsWith($currentRoute, 'chat.') ? 'primary' : 'light' }}">
                                        <i data-feather="message-circle"></i> Chat
                                    </div>
                                </a>
                            </li>
                        </ul>

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
