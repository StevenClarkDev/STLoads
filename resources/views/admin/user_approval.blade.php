@extends('layouts.app')
@section('content')
    <div class="container-fluid">
        <div class="row mt-3">
            <div class="container-fluid">
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
                                <!-- <li class="breadcrumb-item">Admin</li> -->
                            </ol>
                        </div>
                    </div>
                </div>
            </div>
            <div class="col-sm-12">
                <div class="card">
                    <div class="card-header pb-0 card-no-border">
                        <h4>User Approval List</h4>
                        <span>Approve or reject users below.</span>
                    </div>
                    <div class="card-body">
                        <div class="table-responsive user-datatable">
                            <table class="table table-striped" id="user-approval-table">
                                <thead>
                                    <tr>
                                        <th>#</th>
                                        <th>Name</th>
                                        <th>Email</th>
                                        <th>Status</th>
                                        <th>Action</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    @foreach($users as $user)
                                        <tr>
                                            <td>{{ $user->id }}</td>
                                            <td>
                                                @if($user->avatar)
                                                    <img class="img-fluid table-avtar" src="{{ asset('storage/' . $user->avatar) }}"
                                                        alt="" style="width:32px;height:32px;border-radius:50%;margin-right:8px;">
                                                @endif
                                                {{ $user->name }}
                                            </td>
                                            <td>{{ $user->email }}</td>
                                            <td>
                                                <span
                                                    class="badge {{ $user->status == 'approved' ? 'badge-success' : 'badge-warning' }}">
                                                    {{ ucfirst($user->status) }}
                                                </span>
                                            </td>
                                            <td>
                                                <a href="{{ route('user.profile', $user->id) }}">profile</a>
                                                <form action="#" method="POST" style="display:inline;">
                                                    @csrf
                                                    <button type="submit" class="btn btn-success btn-sm" {{ $user->status == 'approved' ? 'disabled' : '' }}>Approve</button>
                                                </form>
                                                <form action="#" method="POST" style="display:inline;">
                                                    @csrf
                                                    <button type="submit" class="btn btn-danger btn-sm" {{ $user->status == 'rejected' ? 'disabled' : '' }}>Reject</button>
                                                </form>
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