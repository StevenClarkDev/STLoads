@extends('auth.app')
@section('content')
    <div class="card p-5 rounded shadow my-4" style="max-width: 500px; width: 100%;">
        <div class="text-center mb-4">
            <img src="../assets/images/stloads/logo-bg_none-small.png" alt="Load Board Logo" style="max-width: 40%;">
        </div>

        <form action="{{ route('otp') }}" method="GET" class="row g-3">
            @csrf

            <h4>Forgot Password</h4>
            <p class="text-muted">Enter your email to receive an OTP</p>

            <div class="col-12 position-relative">
                <label>Email</label>
                <div class="input-group">
                    <input type="hidden" name="id" value="{{ $role->id }}">
                    <input id="email" class="form-control pe-5 rounded-2" type="email" name="email"
                        placeholder="you@example.com" required>
                    <i id="email-icon"
                        class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                </div>
            </div>

            <!-- Submit Button -->
            <div class="col-12 text-center mt-4">
                <button type="submit" class="btn btn-primary w-50">Send OTP</button>
                <p class="mt-3"><a href="{{ route('login', ['id' => $role->id]) }}">Back to Sign In</a></p>
            </div>

            <!-- Laravel Feedback -->
            @if ($errors->any())
                <div class="col-12 text-danger text-center mt-2">
                    {{ $errors->first() }}
                </div>
            @endif

            @if (session('success'))
                <div class="col-12 text-success text-center mt-2">
                    {{ session('success') }}
                </div>
            @endif
        </form>
    </div>
@endsection
