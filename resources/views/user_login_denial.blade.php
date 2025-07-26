@extends('auth.app')

@section('content')
    <div class="container d-flex justify-content-center align-items-center my-5">
        <div class="card p-5 rounded shadow w-100" style="max-width: 500px;">
            <div class="text-center mb-4">
                <img src="{{ asset('assets/images/stloads/logo-bg_none-small.png') }}" alt="Load Board Logo" class="img-fluid" style="max-width: 150px;">
            </div>

            @if ($user->status == 0)
                <div class="alert alert-warning text-center">
                    <h4 class="alert-heading">Pending</h4>
                    <p>You have not filled the onboarding form yet. Please complete it to proceed.</p>
                </div>
                <div class="d-flex justify-content-center gap-2">
                    <a href="{{ route('onboarding-form', $user->id) }}" class="btn btn-primary">Complete Onboarding</a>
                    <a href="{{ route('login', ['id' => $role->id]) }}" class="btn btn-outline-secondary">Cancel</a>
                </div>

            @elseif ($user->status == 3)
                <div class="alert alert-info text-center">
                    <h4 class="alert-heading">KYC Pending</h4>
                    <p>Your KYC verification is pending. Please wait for approval.</p>
                </div>
                <div class="text-center">
                    <a href="{{ route('login', ['id' => $role->id]) }}" class="btn btn-outline-secondary">Back</a>
                </div>

            @elseif ($user->status == 4)
                <div class="alert alert-danger text-center">
                    <h4 class="alert-heading">Email Not Verified</h4>
                    <p>Your email has not been verified yet. Please check your inbox.</p>
                </div>
                <div class="text-center">
                    <a href="{{ route('login', ['id' => $role->id]) }}" class="btn btn-outline-secondary">Back</a>
                </div>

            @elseif ($user->status == 2)
                <div class="alert alert-danger text-center">
                    <h4 class="alert-heading">Rejected</h4>
                    <p>Your account has been rejected. Please contact support for more information.</p>
                </div>
                <div class="text-center">
                    <a href="{{ route('login', ['id' => $role->id]) }}" class="btn btn-outline-secondary">Back</a>
                </div>
            @endif
        </div>
    </div>
@endsection
