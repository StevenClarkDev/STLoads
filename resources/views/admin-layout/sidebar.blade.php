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
                                        <i data-feather="user-check"></i> Brokers
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
                                <a href="{{ url('admin-manage-loads') }}">
                                    <div
                                        class="btn btn-{{ Str::startsWith($currentRoute, 'admin-manage-loads.') ? 'primary' : 'light' }}">
                                        <i data-feather="truck"></i> Manage Loads
                                    </div>
                                </a>
                            </li>
                            {{-- <li class="nav-item mb-2">
                                <a href="{{ url('chat') }}">
                                    <div
                                        class="btn btn-{{ Str::startsWith($currentRoute, 'chat.') ? 'primary' : 'light' }}">
                                        <i data-feather="message-circle"></i> Chat
                                    </div>
                                </a>
                            </li> --}}
                        </ul>
                        @php
                            use Illuminate\Support\Str;

                            $isMasterActive =
                                Str::startsWith($currentRoute, 'load_types') ||
                                Str::startsWith($currentRoute, 'equipments') ||
                                Str::startsWith($currentRoute, 'commodity_types') ||
                                Str::startsWith($currentRoute, 'locations');
                        @endphp

                        <ul class="nav flex-column my-2">
                            <li class="nav-item mb-2">
                                <button
                                    class="btn btn-{{ $isMasterActive ? 'primary' : 'light' }} w-100 d-flex align-items-center justify-content-start"
                                    type="button" data-bs-toggle="collapse" data-bs-target="#masterMenu"
                                    aria-expanded="{{ $isMasterActive ? 'true' : 'false' }}">
                                    <i data-feather="settings" class="me-2"></i> Master Pages
                                </button>
                                <div class="collapse {{ $isMasterActive ? 'show' : '' }} mt-2" id="masterMenu">
                                    <ul class="nav flex-column ms-2">
                                        <li class="nav-item mb-2">
                                            <a href="{{ route('load_types.index') }}" class="w-100">
                                                <div
                                                    class="btn btn-{{ Str::startsWith($currentRoute, 'load_types') ? 'primary' : 'light' }} w-100 text-start">
                                                    <i data-feather="layers" class="me-2"></i> Load Types
                                                </div>
                                            </a>
                                        </li>
                                        <li class="nav-item mb-2">
                                            <a href="{{ route('equipments.index') }}" class="w-100">
                                                <div
                                                    class="btn btn-{{ Str::startsWith($currentRoute, 'equipments') ? 'primary' : 'light' }} w-100 text-start">
                                                    <i data-feather="tool" class="me-2"></i> Equipments
                                                </div>
                                            </a>
                                        </li>
                                        <li class="nav-item mb-2">
                                            <a href="{{ route('commodity_types.index') }}" class="w-100">
                                                <div
                                                    class="btn btn-{{ Str::startsWith($currentRoute, 'commodity_types') ? 'primary' : 'light' }} w-100 text-start">
                                                    <i data-feather="package" class="me-2"></i> Commodity Types
                                                </div>
                                            </a>
                                        </li>
                                        <li class="nav-item mb-2">
                                            <a href="{{ route('locations.index') }}" class="w-100">
                                                <div
                                                    class="btn btn-{{ Str::startsWith($currentRoute, 'locations') ? 'primary' : 'light' }} w-100 text-start">
                                                    <i data-feather="map-pin" class="me-2"></i> Locations
                                                </div>
                                            </a>
                                        </li>
                                    </ul>
                                </div>
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

<link rel="stylesheet" type="text/css" href="{{ url('assets/css/vendors/feather-icon.css') }}">
<script src="https://unpkg.com/feather-icons"></script>
<script>
    feather.replace();
</script>