@extends('admin.app')
@section('content')
    <div class="container-fluid p-0 m-0 min-vh-100 d-flex">
        <div class="row g-0 flex-grow-1 mt-3">
            <!-- Sidebar Column -->
            <div class="col-xl-3 box-col-6 ps-3 pt-3">
                <div class="md-sidebar">
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
                                                <div class="btn btn-light"><i data-feather="send"></i>Freight Forwarders
                                                </div>
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

            <!-- Main Content Column -->
            <div class="col-xl-9 box-col-6 p-3">
                <div class="card h-100">
                    <div class="card-body p-0">
                        <div class="card">
                            <div class="card-header pb-0 card-no-border">
                                <h4>Brookers List</h4>
                                <span>See registered users below.</span>
                            </div>
                            <div class="card-body">
                                <div class="table-responsive">
                                    <table class="table table-striped w-100" id="user-approval-table">
                                        <thead>
                                            <tr>
                                                <th style="width: 5%">S No.</th>
                                                <th style="width: 20%">Name</th>
                                                <th style="width: 25%">Email</th>
                                                <th style="width: 15%">Role</th>
                                                <th style="width: 15%">Date</th>
                                                <th style="width: 20%">Action</th>
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
                                                    <td>User Role</td>
                                                    <td>7th July 2025</td>
                                                    <td class="d-flex gap-2">
                                                        <a href="{{ route('user.profile', $user->id) }}"
                                                            class="btn btn-info btn-sm flex-grow-1">Profile</a>
                                                        <button type="button" class="btn btn-success btn-sm flex-grow-1"
                                                            data-bs-toggle="modal"
                                                            data-bs-target="#contactModal">Contact</button>
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

    <!-- Contact Modal -->
    <div class="modal fade" id="contactModal" tabindex="-1" aria-labelledby="contactModalLabel" aria-hidden="true">
        <div class="modal-dialog modal-dialog-centered">
            <div class="modal-content border border-primary">
                <div class="modal-header">
                    <h5 class="modal-title">Contact User</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <form>
                        <div class="mb-3">
                            <label for="userEmail" class="form-label">Email</label>
                            <input type="email" class="form-control" id="userEmail" value="{{ $user->email ?? '' }}"
                                readonly>
                        </div>
                        <div class="mb-3">
                            <label for="userContact" class="form-label">Contact</label>
                            <input type="text" class="form-control" id="userContact" value="{{ $user->contact ?? 'N/A' }}"
                                readonly>
                        </div>
                    </form>
                    <div class="d-flex justify-content-center gap-3">
                        <a href="#" class="text-decoration-none" data-bs-toggle="modal"
                            data-bs-target="#serviceUnavailableModal">
                            <i class="fab fa-whatsapp text-success" style="font-size: 1.5rem;"></i>
                        </a>
                        <a href="#" class="text-decoration-none" data-bs-toggle="modal"
                            data-bs-target="#serviceUnavailableModal">
                            <i class="fab fa-telegram text-primary" style="font-size: 1.5rem;"></i>
                        </a>
                        <a href="#" class="text-decoration-none" data-bs-toggle="modal"
                            data-bs-target="#serviceUnavailableModal">
                            <i class="fab fa-facebook-messenger text-info" style="font-size: 1.5rem;"></i>
                        </a>
                        <a href="#" class="text-decoration-none" data-bs-toggle="modal"
                            data-bs-target="#serviceUnavailableModal">
                            <i class="fab fa-instagram text-danger" style="font-size: 1.5rem;"></i>
                        </a>
                        <a href="#" class="text-decoration-none" data-bs-toggle="modal"
                            data-bs-target="#serviceUnavailableModal">
                            <i class="fas fa-envelope text-warning" style="font-size: 1.5rem;"></i>
                        </a>
                    </div>
                </div>
            </div>
        </div>
    </div>
    </div>

    <!-- Service Unavailable Modal -->
    <div class="modal fade" id="serviceUnavailableModal" tabindex="-1" aria-labelledby="serviceUnavailableLabel"
        aria-hidden="true">
        <div class="modal-dialog modal-dialog-centered">
            <div class="modal-content border border-danger">
                <div class="modal-header">
                    <h5 class="modal-title" id="serviceUnavailableLabel">Service Unavailable</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body text-center">
                    <p>Sorry, this service is not available right now.</p>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">OK</button>
                </div>
            </div>
        </div>
    </div>
@endsection

@push('styles')
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.0.0/css/all.min.css">
@endpush