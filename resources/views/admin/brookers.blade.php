@extends('admin.app')
@section('content')
    <div class="card d-flex align-items-center justify-content-center min-vh-100 p-3 m-3 bg-transparent">
        <div class="row">
            <!-- <div class="container-fluid">
                <div class="page-title">
                    <div class="row">
                        <div class="col-6">
                            <h4>User Approval</h4>
                        </div>
                        <div class="col-6">
                            <ol class="breadcrumb">
                                <li class="breadcrumb-item"><a href="index.html">
                                        <svg class="stroke-icon">
                                            <use href="../assets/svg/icon-sprite.svg#stroke-home"></use>
                                        </svg></a></li>
                                <li class="breadcrumb-item active">User Approval</li>
                                <li class="breadcrumb-item">Admin</li>
                            </ol>
                        </div>
                    </div>
                </div>
            </div> -->
            <div class="col-xl-3 box-col-6 pe-0">
                <div class="md-sidebar"><a class="btn btn-primary md-sidebar-toggle" href="javascript:void(0)">file
                        filter</a>
                    <div class="md-sidebar-aside job-left-aside custom-scrollbar">
                        <div class="file-sidebar">
                            <div class="card">
                                <div class="card-body">
                                    <ul>
                                        <li>
                                            <a href="{{ route('user_approval') }}">
                                                <div class="btn btn-light"><i data-feather="home"> </i>Home </div>
                                            </a>
                                        </li>
                                        <li>
                                            <a href="{{ route('carriers') }}">
                                                <div class="btn btn-light"><i data-feather="truck"></i>Carriers </div>
                                            </a>
                                        </li>
                                        <li>
                                            <a href="{{ route('shippers') }}">
                                                <div class="btn btn-light"><i data-feather="box"></i>Shippers </div>
                                            </a>
                                            </li>
                                            <li>
                                                <a href="{{ route('brookers') }}">
                                                <div class="btn btn-primary"><i data-feather="user-check"></i>Brookers </div>
                                            </a>
                                        </li>
                                        <li>
                                            <a href="{{ route(name: 'freight_forwarders') }}">
                                                <div class="btn btn-light"><i data-feather="send"></i>Freight Forwarders </div>
                                            </a>
                                        </li>
                                    </ul>
                                    <hr>
                                    <ul>
                                        <li>
                                            <div class="btn btn-outline-primary"><i data-feather="database"> </i>Storage
                                            </div>
                                            <div class="m-t-15">
                                                <div class="progress sm-progress-bar mb-1">
                                                    <div class="progress-bar bg-primary" role="progressbar"
                                                        style="width: 25%" aria-valuenow="25" aria-valuemin="0"
                                                        aria-valuemax="100"></div>
                                                </div>
                                                <p>25 GB of 100 GB used</p>
                                            </div>
                                        </li>
                                    </ul>
                                    <hr>
                                    <ul>
                                        <li>
                                            <div class="btn btn-outline-secondary"><i data-feather="log-out"> </i>Logout
                                            </div>
                                        </li>
                                    </ul>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            <div class="col-xl-9 box-col-6 pe-0">
                <div class="file-content">
                    <div class="card">
                        <div class="container-fluid">
                            <div class="row mt-3">
                                <div class="col-sm-12">
                                    <div class="card">
                                        <div class="card-header pb-0 card-no-border">
                                            <h4>Brookers List</h4>
                                            <span>See registered users below.</span>
                                        </div>
                                        <div class="card-body">
                                            <div class="table-responsive user-datatable">
                                                <table class="table table-striped" id="user-approval-table">
                                                    <thead>
                                                        <tr>
                                                            <th>S No.</th>
                                                            <th>Name</th>
                                                            <th>Email</th>
                                                            <th>Role</th>
                                                            <th>Date</th>
                                                            <th>Action</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        @foreach($users as $user)
                                                            <tr>
                                                                <td>{{ $user->id }}</td>
                                                                <td>
                                                                    @if($user->avatar)
                                                                        <img class="img-fluid table-avtar"
                                                                            src="{{ asset('storage/' . $user->avatar) }}" alt=""
                                                                            style="width:32px;height:32px;border-radius:50%;margin-right:8px;">
                                                                    @endif
                                                                    {{ $user->name }}
                                                                </td>
                                                                <td>{{ $user->email }}</td>
                                                                <!-- <td>
                                                                                                                                                                                                                                                @php
                                                                                                                                                                                                                                                    $roleColors = [
                                                                                                                                                                                                                                                        'carrier' => 'badge-primary',
                                                                                                                                                                                                                                                        'brooker' => 'badge-info',
                                                                                                                                                                                                                                                        'shipper' => 'badge-success',
                                                                                                                                                                                                                                                        'freight forwarder' => 'badge-warning',
                                                                                                                                                                                                                                                    ];
                                                                                                                                                                                                                                                    $role = strtolower($user->role);
                                                                                                                                                                                                                                                    $badgeClass = $roleColors[$role] ?? 'badge-secondary';
                                                                                                                                                                                                                                                @endphp
                                                                                                                                                                                                                                                <span class="badge {{ $badgeClass }}">
                                                                                                                                                                                                                                                    {{ ucfirst($user->role) }}
                                                                                                                                                                                                                                                </span>
                                                                                                                                                                                                                                            </td> -->
                                                                <td>User Role</td>
                                                                <td>7th July 2025</td>
                                                                <td>
                                                                    <a href="{{ route('user.profile', $user->id) }}"
                                                                        class="btn btn-info btn-sm">Profile</a>
                                                                    <button type="button" class="btn btn-success btn-sm"
                                                                        data-bs-toggle="modal"
                                                                        data-bs-target="#approveModal">Contact</button>
                                                                </td>
                                                            </tr>
                                                        @endforeach
                                                    </tbody>
                                                </table>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

    </div>


    <script src="https://cdn.jsdelivr.net/npm/sweetalert2@11"></script>

@endsection