@extends('layout.app')
@section('content')
    <!-- Main Content Column -->
    <div class="col-xl-12 box-col-6 p-3">
        <div class="card">
            <div class="card-body p-0">
                <div class="card mx-3">
                    <div class="card-header pb-0 card-no-border">
                        <h4>{{ $role->name }}s List</h4>
                        <span>See registered users below.</span>
                    </div>
                    <div class="card-body">
                        <div class="table-responsive user-datatable">
                            <div style="max-height:500px; min-height:210px; overflow-y:auto;">
                                <table class="table table-striped w-100" id="user-approval-table">
                                    <thead style="position: sticky; top: 0; background: #fff; z-index: 2;">
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
                                        @if ($users->isEmpty())
                                            <tr>
                                                <td colspan="6" class="text-center">No data available.</td>
                                            </tr>
                                        @endif
                                        @foreach ($users as $index => $user)
                                            <tr>
                                                <td>{{ $index + 1 }}</td>
                                                <td>
                                                    @if ($user->avatar)
                                                        <img class="img-fluid table-avtar"
                                                            src="{{ asset('storage/' . $user->avatar) }}" alt=""
                                                            style="width:32px;height:32px;border-radius:50%;margin-right:8px;">
                                                    @endif
                                                    {{ $user->name }}
                                                </td>
                                                <td>{{ $user->email }}</td>
                                                <td>
                                                    @if (!empty($user->getRoleNames()))
                                                        @foreach ($user->getRoleNames() as $v)
                                                            {{ $v }}
                                                        @endforeach
                                                    @endif
                                                </td>
                                                <td>{{ $user->created_at->format('jS F Y') }}</td>
                                                <td class="d-flex gap-2">
                                                    <a href="{{ route('user.profile', $user->id) }}"
                                                        class="btn btn-info btn-sm flex-grow-1">Profile</a>
                                                    <button type="button" class="btn btn-success btn-sm flex-grow-1"
                                                        data-bs-toggle="modal" data-bs-target="#contactModal">Contact</button>
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
@endsection
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
                            <input type="text" class="form-control" id="userContact"
                                value="{{ $user->contact ?? 'N/A' }}" readonly>
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

    @push('styles')
        <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.0/css/all.min.css"
            integrity="sha512-..." crossorigin="anonymous" referrerpolicy="no-referrer" />
    @endpush

    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/5.15.4/css/all.min.css"
        integrity="sha512-dyZt9u+0A2ZyWOKGqhg9Ulmgwv9z5s8EKz6eS8dDXCzZsAJ2w5PZg6SLYmcm+3b0q6Iq2nX9mthV9Ic2uZlUgQ=="
        crossorigin="anonymous" referrerpolicy="no-referrer" />