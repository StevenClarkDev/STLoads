@extends('layout.app')

@section('content')
<div class="col overflow-auto px-3">
    <div class="row">
        <div class="col-xl-12 mt-3">
            <div class="card mx-3">
                <div class="card-header d-flex justify-content-between align-items-center">
                    <h5 class="mb-0">Edit Profile</h5>
                    <a href="{{ route('profile', $user) }}" class="btn btn-secondary btn-sm">
                        <i class="fa fa-arrow-left"></i> Back to Profile
                    </a>
                </div>
                <div class="card-body">
                    @if (count($errors) > 0)
                        <div class="alert alert-danger">
                            <strong>Whoops!</strong> There were some problems with your input.<br><br>
                            <ul>
                                @foreach ($errors->all() as $error)
                                    <li>{{ $error }}</li>
                                @endforeach
                            </ul>
                        </div>
                    @endif

                    @if(session('success'))
                        <div class="alert alert-success">
                            {{ session('success') }}
                        </div>
                    @endif

                    <form method="POST" action="{{ route('profile.update', $user) }}">
                        @csrf

                        <div class="row g-3">
                            <!-- Basic Information -->
                            <div class="col-12">
                                <h6 class="text-primary mb-3">Basic Information</h6>
                            </div>

                            <div class="col-md-6">
                                <label class="form-label">Name <span class="text-danger">*</span></label>
                                <input type="text" name="name" class="form-control" value="{{ old('name', $user->name) }}" required>
                            </div>

                            <div class="col-md-6">
                                <label class="form-label">Email <span class="text-danger">*</span></label>
                                <input type="email" name="email" class="form-control" value="{{ old('email', $user->email) }}" required>
                            </div>

                            <div class="col-md-6">
                                <label class="form-label">Phone Number</label>
                                <input type="text" name="phone_no" class="form-control" value="{{ old('phone_no', $user->phone_no) }}">
                            </div>

                            <div class="col-md-6">
                                <label class="form-label">Role</label>
                                <input type="text" class="form-control" value="{{ $role ? $role->name : 'N/A' }}" readonly disabled>
                            </div>

                            <div class="col-12">
                                <label class="form-label">Address</label>
                                <textarea name="address" class="form-control" rows="2">{{ old('address', $user->address) }}</textarea>
                            </div>

                            @if($role && in_array($role->name, ['Carrier', 'Shipper', 'Broker', 'Freight Forwarder']))
                            <!-- Company Information -->
                            <div class="col-12 mt-4">
                                <h6 class="text-primary mb-3">Company Information</h6>
                            </div>

                            <div class="col-md-6">
                                <label class="form-label">Company Name</label>
                                <input type="text" name="company_name" class="form-control" value="{{ old('company_name', $user->company_name) }}">
                            </div>

                            @if($role->name === 'Carrier')
                                <div class="col-md-6">
                                    <label class="form-label">DOT Number</label>
                                    <input type="text" name="dot_no" class="form-control" value="{{ old('dot_no', $user->dot_no) }}">
                                </div>

                                <div class="col-md-6">
                                    <label class="form-label">MC Number</label>
                                    <input type="text" name="mc_no" class="form-control" value="{{ old('mc_no', $user->mc_no) }}">
                                </div>
                            @endif

                            @if($role->name === 'Broker')
                                <div class="col-md-6">
                                    <label class="form-label">MC/CBSA/USDOT Number</label>
                                    <input type="text" name="mc_cbsa_usdot_no" class="form-control" value="{{ old('mc_cbsa_usdot_no', $user->mc_cbsa_usdot_no) }}">
                                </div>

                                <div class="col-md-6">
                                    <label class="form-label">UCR/HCC Number</label>
                                    <input type="text" name="ucr_hcc_no" class="form-control" value="{{ old('ucr_hcc_no', $user->ucr_hcc_no) }}">
                                </div>
                            @endif
                            @endif

                            <!-- Password Change Section -->
                            <div class="col-12 mt-4">
                                <h6 class="text-primary mb-3">Change Password (Optional)</h6>
                                <p class="text-muted small">Leave blank if you don't want to change your password</p>
                            </div>

                            <div class="col-md-6">
                                <label class="form-label">New Password</label>
                                <div class="position-relative">
                                    <input type="password" name="password" class="form-control pe-5" placeholder="Enter new password">
                                    <i class="fas fa-eye pwd-toggle text-muted position-absolute top-50 end-0 translate-middle-y me-3" style="cursor:pointer" title="Show/hide password"></i>
                                </div>
                            </div>

                            <div class="col-md-6">
                                <label class="form-label">Confirm New Password</label>
                                <div class="position-relative">
                                    <input type="password" name="password_confirmation" class="form-control pe-5" placeholder="Confirm new password">
                                    <i class="fas fa-eye pwd-toggle text-muted position-absolute top-50 end-0 translate-middle-y me-3" style="cursor:pointer" title="Show/hide password"></i>
                                </div>
                            </div>

                            <!-- Submit Button -->
                            <div class="col-12 mt-4">
                                <button type="submit" class="btn btn-primary">
                                    <i class="fa fa-save"></i> Update Profile
                                </button>
                                <a href="{{ route('profile', $user) }}" class="btn btn-secondary">
                                    <i class="fa fa-times"></i> Cancel
                                </a>
                            </div>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    </div>
</div>

<script>
    // Password show/hide toggle
    document.querySelectorAll('.pwd-toggle').forEach(function(icon) {
        icon.addEventListener('click', function() {
            var input = this.parentElement.querySelector('input');
            if (input) {
                input.type = input.type === 'password' ? 'text' : 'password';
                this.classList.toggle('fa-eye');
                this.classList.toggle('fa-eye-slash');
            }
        });
    });
</script>
@endsection
