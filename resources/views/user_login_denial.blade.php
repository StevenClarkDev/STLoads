@extends('auth.app')
@section('content')
    <div class="card p-5 rounded shadow my-4" style="max-width: 500px; width: 100%;">
        <div class="text-center mb-4">
            <img src="../assets/images/stloads/logo-bg_none-small.png" alt="Load Board Logo" style="max-width: 40%;">
        </div>
        @if ($user->status == 0)
            <h4>Pending</h4>
            <p class="text-muted">You have not filled the onboarding form yet. Please complete it to proceed.</p>
            <a href="{{ route('onboarding-form', $user->id) }}" class="btn btn-primary me-sm-3 me-1">Onboarding Form</a>
            <a href="{{ route('login', ['id' => $role->id]) }}" type="back" class="btn btn-outline-secondary">Cancel</a>
        @elseif ($user->status == 3)
            <h4>KYC Pending</h4>
            <p class="text-muted">Your KYC verification is pending. Please wait for approval.</p>
            <a href="{{ route('login', ['id' => $role->id]) }}" type="back" class="btn btn-outline-secondary">Back</a>
        @elseif ($user->status == 4)
            <h4>Email Not Verified</h4>
            <p class="text-muted">Your Email Not is Verified.</p>
            <a href="{{ route('login', ['id' => $role->id]) }}" type="back" class="btn btn-outline-secondary">Back</a>
        @elseif ($user->status == 2)
            <h4>Rejected</h4>
            <p class="text-muted">Your account has been rejected. Please contact support for more information.</p>
            <a href="{{ route('login', ['id' => $role->id]) }}" type="back" class="btn btn-outline-secondary">Back</a>
        @endif
    </div>
@endsection
