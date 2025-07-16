@extends('auth.app')
@section('content')
    <div class="card p-5 rounded shadow my-4" style="max-width: 1100px; width: 100%;">
        <div class="text-center mb-4"> <img src="../assets/images/stloads/logo-bg_none-small.png" alt="Load Board Logo"
                style="max-width: 30%;"> </div>
        <form class="row g-3" action="{{ route('register') }}" method="POST">
            @csrf

            <h4>Sign Up</h4>
            <p class="text-muted">Enter your details to create an account</p>

            {{-- Email --}}
            <div class="col-md-6 position-relative">
                <label>Email</label>
                <div class="input-group">
                    <input id="email" class="form-control pe-5 rounded-2" type="email" name="email"
                        placeholder="you@example.com" required>
                    <i id="email-icon"
                        class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                </div>
            </div>

            {{-- Name --}}
            <div class="col-md-6 position-relative">
                <label>Name</label>
                <div class="input-group">
                    <input id="name" class="form-control pe-5 rounded-2" type="text" name="name"
                        placeholder="Full Name" required>
                    <i id="name-icon"
                        class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                </div>
            </div>

            {{-- Password --}}
            <div class="col-md-6 position-relative">
                <label>Password</label>
                <div class="input-group">
                    <input id="password" class="form-control pe-5 rounded-2" type="password" name="password"
                        placeholder="********" required>
                    <i id="password-icon"
                        class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                </div>
                <small id="password-error" class="text-danger d-none">Password must be at least 8 characters and
                    contain a letter and number.</small>
            </div>

            {{-- Role --}}
            <div class="col-md-6 position-relative">
                <label>Role</label>
                <div class="input-group">
                    <input id="role" class="form-control pe-5 rounded-2" type="text" name="role"
                        placeholder="Carrier / Broker" required>
                </div>
            </div>

            {{-- Confirm Password --}}
            <div class="col-md-6 position-relative"> <label>Confirm Password</label>
                <div class="input-group"> <input id="confirm-password" class="form-control pe-5 rounded-2" type="password"
                        name="password_confirmation" placeholder="********" required> <i id="confirm-password-icon"
                        class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                </div>
            </div>
            {{-- DOB + Gender --}}
            <div class="col-md-6 d-flex gap-2">
                <div class="flex-fill position-relative">
                    <label>Date of Birth</label>
                    <input class="form-control pe-5 rounded-2 text-muted" type="date" name="dob" required>
                </div>
                {{-- Gender Dropdown --}}
                <div class="flex-fill">
                    <label>Gender</label>
                    <select id="gender" class="form-control pe-5 rounded-2 text-muted" name="gender" required>
                        <option disabled selected>Select Gender</option>
                        <option>Male</option>
                        <option>Female</option>
                        <option>Other</option>
                    </select>
                </div>

            </div>

            {{-- CNIC --}}
            <div class="col-md-6 position-relative">
                <label>CNIC</label>
                <div class="input-group">
                    <input id="cnic" class="form-control pe-5 rounded-2" type="text" name="cnic"
                        placeholder="e.g., 42101XXXXXXXXX" required>
                    <i id="cnic-icon"
                        class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                </div>
            </div>

            {{-- Address --}}
            <div class="col-md-6">
                <label>Address</label>
                <input class="form-control" type="text" name="address" placeholder="Complete Address" required>
            </div>

            {{-- Submit --}}
            <div class="col-12 text-center mt-4">
                <button type="submit" class="btn btn-primary w-50">Sign Up</button>
                <p class="mt-3 text-muted">Already have an account? <a href="{{ route('login') }}">Sign In</a>
                </p>
            </div>

            {{-- Laravel error/success messages --}}
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
