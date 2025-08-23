@extends('auth.app')

@section('content')
    <div class="container d-flex justify-content-center align-items-center my-5">
        <div class="card p-5 rounded shadow w-100" style="max-width: 500px;">
            <div class="text-center mb-4">
                <img src="{{ asset('assets/images/stloads/logo-bg_none-small.png') }}" alt="Load Board Logo"
                    class="img-fluid" style="max-width: 150px;">
            </div>

            {{-- Status Handling --}}
            @if ($user->status == 0)
                <div class="text-center">
                    <h4 class="mb-3">Pending</h4>
                    <p class="text-muted">You have not filled the onboarding form yet. Please complete it to proceed.</p>
                    <div class="d-flex justify-content-center gap-2">
                        <a href="{{ route('onboarding-form', $user->id) }}" class="btn btn-primary">Complete Onboarding</a>
                        <a href="{{ route('normal-login', ['id' => $role->id]) }}" class="btn btn-outline-secondary">Cancel</a>
                    </div>
                </div>

            @elseif ($user->status == 3)
                <div class="text-center">
                    <h4 class="mb-3">KYC Pending</h4>
                    <p class="text-muted">Your KYC verification is pending. Please wait for approval.</p>
                    <div class="text-center">
                        <a href="{{ route('normal-login', ['id' => $role->id]) }}" class="btn btn-outline-secondary">Back</a>
                    </div>
                </div>

            @elseif ($user->status == 4)
                <div class="text-center">
                    <h4 class="mb-3">Email Not Verified</h4>
                    <p class="text-muted">Your email has not been verified yet. Please check your inbox.</p>
                    <div class="text-center">
                        <a href="{{ route('normal-login', ['id' => $role->id]) }}" class="btn btn-outline-secondary">Back</a>
                    </div>
                </div>

            @elseif ($user->status == 2)
                <div class="text-center">
                    <h4 class="mb-3 text-danger">Rejected</h4>
                    <p class="text-muted">Your account has been rejected. Please contact support for more information.</p>
                    <div class="text-center">
                        <a href="{{ route('normal-login', ['id' => $role->id]) }}" class="btn btn-outline-secondary">Back</a>
                    </div>
                </div>
            @endif
        </div>
    </div>
@endsection