@extends('admin-layout.app')
@section('content')
<div class="row justify-content-center py-4">
    <div class="col-md-6 col-lg-5">
        <div class="card shadow-sm">
            <div class="card-header">
                <h5 class="mb-0">Change Password</h5>
            </div>
            <div class="card-body">

                @if (session('success'))
                    <div class="alert alert-success">{{ session('success') }}</div>
                @endif
                @if ($errors->any())
                    <div class="alert alert-danger">{{ $errors->first() }}</div>
                @endif

                <form action="{{ route('admin.change-password.update') }}" method="POST">
                    @csrf

                    <div class="mb-3">
                        <label class="form-label">Current Password <span class="text-danger">*</span></label>
                        <div class="position-relative">
                            <input type="password" class="form-control pe-5" name="current_password" required>
                            <i class="fas fa-eye pwd-toggle text-muted position-absolute top-50 end-0 translate-middle-y me-3" style="cursor:pointer" title="Show/hide password"></i>
                        </div>
                    </div>

                    <div class="mb-3">
                        <label class="form-label">New Password <span class="text-danger">*</span></label>
                        <div class="position-relative">
                            <input type="password" class="form-control pe-5" name="password"
                                   placeholder="Min. 8 characters" required minlength="8">
                            <i class="fas fa-eye pwd-toggle text-muted position-absolute top-50 end-0 translate-middle-y me-3" style="cursor:pointer" title="Show/hide password"></i>
                        </div>
                    </div>

                    <div class="mb-4">
                        <label class="form-label">Confirm New Password <span class="text-danger">*</span></label>
                        <div class="position-relative">
                            <input type="password" class="form-control pe-5" name="password_confirmation" required>
                            <i class="fas fa-eye pwd-toggle text-muted position-absolute top-50 end-0 translate-middle-y me-3" style="cursor:pointer" title="Show/hide password"></i>
                        </div>
                    </div>

                    <button type="submit" class="btn btn-primary w-100">Update Password</button>
                </form>

            </div>
        </div>
    </div>
</div>
@endsection
