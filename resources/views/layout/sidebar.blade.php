{{-- <div class="col-xl-3 box-col-6 ps-3 pt-3 h-100">
    <div class="md-sidebar h-100">
        <div class="md-sidebar-aside job-left-aside custom-scrollbar h-100">
            <div class="file-sidebar h-100">
                <div class="card h-100">
                    <div class="card-body"> --}}
<div class="col-xl-3 box-col-6 ps-3 pt-3">
    <div class="md-sidebar">
        <div class="md-sidebar-aside job-left-aside custom-scrollbar">
            <div class="file-sidebar">
                <div class="card">
                    <div class="card-body">
                        @php
                            $currentRoute = Route::currentRouteName();
                        @endphp
                        <ul>
                            <li>
                                <a href="{{ route('user_approval') }}">
                                    <div class="btn btn-{{ $currentRoute == 'user_approval' ? 'primary' : 'light' }}"><i data-feather="home"></i>Home</div>
                                </a>
                            </li>
                            <li>
                                <a href="{{ route('users_by_role', 3) }}">
                                    <div class="btn btn-{{ ($currentRoute == 'users_by_role' && request()->route('id') == 3) ? 'primary' : 'light' }}"><i data-feather="truck"></i>Carriers</div>
                                </a>
                            </li>
                            <li>
                                <a href="{{ route('users_by_role', 2) }}">
                                    <div class="btn btn-{{ ($currentRoute == 'users_by_role' && request()->route('id') == 2) ? 'primary' : 'light' }}"><i data-feather="box"></i>Shippers</div>
                                </a>
                            </li>
                            <li>
                                <a href="{{ route('users_by_role', 4) }}">
                                    <div class="btn btn-{{ ($currentRoute == 'users_by_role' && request()->route('id') == 4) ? 'primary' : 'light' }}"><i data-feather="user-check"></i>Brookers</div>
                                </a>
                            </li>
                            <li>
                                <a href="{{ route('users_by_role', 5) }}">
                                    <div class="btn btn-{{ ($currentRoute == 'users_by_role' && request()->route('id') == 5) ? 'primary' : 'light' }}"><i data-feather="send"></i>Freight Forwarders</div>
                                </a>
                            </li>
                            <li>
                                <a href="{{ url('roles') }}">
                                    <div class="btn btn-{{ $currentRoute == 'roles.index' ? 'primary' : 'light' }}"><i data-feather="shield"></i>Roles</div>
                                </a>
                            </li>
                        </ul>
                        {{-- <hr>
                        <ul>
                            <li>
                                <div class="btn btn-outline-primary"><i data-feather="database"></i>Storage
                                </div>
                                <div class="m-t-15">
                                    <div class="progress sm-progress-bar mb-1">
                                        <div class="progress-bar bg-primary" role="progressbar" style="width: 25%"
                                            aria-valuenow="25" aria-valuemin="0" aria-valuemax="100"></div>
                                    </div>
                                    <p>25 GB of 100 GB used</p>
                                </div>
                            </li>
                        </ul> --}}
                        <hr>
                        <ul>
                            <li>
                                <form method="POST" action="{{ route('logout') }}" style="display:inline">
                                    @csrf

                                    {{-- <div class="btn btn-outline-secondary"> --}}
                                        <button type="submit" class="btn btn-outline-secondary"><i data-feather="log-out"></i> Logout</button>
                                    {{-- </div> --}}
                                </form>
                                {{-- <a href="{{ route('logout') }}"><i data-feather="log-out"></i>Logout</a> --}}
                            </li>
                        </ul>
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>
