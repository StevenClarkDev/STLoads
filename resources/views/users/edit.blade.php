@extends('admin-layout.app')

@section('content')
<div class="col-12 px-3 py-2">
    <div class="card">
        <div class="card-header d-flex justify-content-between align-items-center">
            <div>
                <h4 class="mb-0">Edit User</h4>
                <span class="text-muted">Update user information and roles</span>
            </div>
            <a class="btn btn-light btn-sm" href="{{ route('users.index') }}">
                <i class="fa fa-arrow-left"></i> Back to Users
            </a>
        </div>

        <div class="card-body">
            @if (count($errors) > 0)
                <div class="alert alert-danger alert-dismissible fade show" role="alert">
                    <strong>Whoops!</strong> There were some problems with your input.
                    <ul class="mb-0 mt-2">
                        @foreach ($errors->all() as $error)
                            <li>{{ $error }}</li>
                        @endforeach
                    </ul>
                    <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>
                </div>
            @endif

            <form method="POST" action="{{ route('users.update', $user->id) }}">
                @csrf
                @method('PUT')

                <div class="row g-3">
                    <!-- Basic Information Section -->
                    <div class="col-12">
                        <h6 class="text-primary mb-3">Basic Information</h6>
                    </div>

                    <div class="col-md-6">
                        <label class="form-label">Name <span class="text-danger">*</span></label>
                        <input type="text" name="name" placeholder="Enter full name" 
                               class="form-control" value="{{ $user->name }}" required>
                    </div>

                    <div class="col-md-6">
                        <label class="form-label">Email <span class="text-danger">*</span></label>
                        <input type="email" name="email" placeholder="Enter email address" 
                               class="form-control" value="{{ $user->email }}" required>
                    </div>

                    <!-- Password Section -->
                    <div class="col-12 mt-4">
                        <h6 class="text-primary mb-3">Password (Optional)</h6>
                        <p class="text-muted small">Leave blank if you don't want to change the password</p>
                    </div>

                    <div class="col-md-6">
                        <label class="form-label">Password</label>
                        <div class="input-group">
                            <input type="password" name="password" placeholder="Enter new password" 
                                   class="form-control password-input">
                            <button class="btn btn-outline-secondary pwd-toggle" type="button">
                                <i class="fas fa-eye"></i>
                            </button>
                        </div>
                    </div>

                    <div class="col-md-6">
                        <label class="form-label">Confirm Password</label>
                        <div class="input-group">
                            <input type="password" name="confirm-password" placeholder="Confirm password" 
                                   class="form-control password-input">
                            <button class="btn btn-outline-secondary pwd-toggle" type="button">
                                <i class="fas fa-eye"></i>
                            </button>
                        </div>
                    </div>

                    <!-- Role Section -->
                    <div class="col-12 mt-4">
                        <h6 class="text-primary mb-3">Role Assignment</h6>
                    </div>

                    <div class="col-md-6">
                        <label class="form-label">User Role(s) <span class="text-danger">*</span></label>
                        <select name="roles[]" class="form-select" size="5" multiple>
                            @foreach ($roles as $value => $label)
                                <option value="{{ $value }}" {{ isset($userRole[$value]) ? 'selected' : ''}}>
                                    {{ $label }}
                                </option>
                            @endforeach
                        </select>
                        <small class="text-muted">Hold Ctrl (Cmd on Mac) to select multiple roles</small>
                    </div>

                    <!-- Submit Buttons -->
                    <div class="col-12 mt-4">
                        <hr>
                        <div class="d-flex gap-2">
                            <button type="submit" class="btn btn-primary">
                                <i class="fa fa-save"></i> Update User
                            </button>
                            <a href="{{ route('users.index') }}" class="btn btn-secondary">
                                <i class="fa fa-times"></i> Cancel
                            </a>
                        </div>
                    </div>
                </div>
            </form>
        </div>
    </div>
</div>

@push('scripts')
<script>
    document.addEventListener('DOMContentLoaded', function() {
        // Password show/hide toggle
        document.querySelectorAll('.pwd-toggle').forEach(function(button) {
            button.addEventListener('click', function() {
                const input = this.closest('.input-group').querySelector('.password-input');
                const icon = this.querySelector('i');
                
                if (input) {
                    if (input.type === 'password') {
                        input.type = 'text';
                        icon.classList.remove('fa-eye');
                        icon.classList.add('fa-eye-slash');
                    } else {
                        input.type = 'password';
                        icon.classList.remove('fa-eye-slash');
                        icon.classList.add('fa-eye');
                    }
                }
            });
        });
    });
</script>
@endpush
@endsection
