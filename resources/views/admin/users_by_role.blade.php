@extends('admin-layout.app')
@section('content')
    <div class="col-12 px-3 py-2">
        <div class="card">
            <div class="card-body p-0">
                <div class="card mx-3">
                    <div class="card-header pb-0 card-no-border">
                        <div class="d-flex justify-content-between align-items-center flex-wrap">
                            <div>
                                <h4>{{ $role->name }}s List</h4>
                                <span>See registered users below.</span>
                            </div>
                            <div class="mb-2" style="min-width: 300px;">
                                <input type="text" id="searchUsersByRole" class="form-control form-control-sm" placeholder="Search by Name, Email, Role...">
                            </div>
                        </div>
                    </div>
                    <div class="card-body">
                        <div class="table-responsive user-datatable">
                            <div style="max-height: 800px; min-height: 210px; overflow-y: auto;">
                                <div class="table-responsive">
                                    <table class="table table-striped align-middle text-nowrap" id="user-approval-table"
                                        style="font-size: 0.875rem;">
                                        <thead class="bg-white" style="position: sticky; top: 0; z-index: 2;">
                                            <tr>
                                                <th>#</th>
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
                                                        @if ($user->image)
                                                            <img src="{{ route('admin.serve-kyc-file', ['path' => $user->image]) }}" alt=""
                                                                style="width: 28px; height: 28px; border-radius: 50%; margin-right: 8px;">
                                                        @else
                                                            <img src="{{ asset('assets/images/user/user.png') }}" alt=""
                                                                style="width: 28px; height: 28px; border-radius: 50%; margin-right: 8px;">
                                                        @endif
                                                        <span class="text-truncate">{{ $user->name }}</span>
                                                    </td>
                                                    <td>{{ $user->email }}</td>
                                                    <td>
                                                        @foreach ($user->getRoleNames() as $v)
                                                            {{ $v }}
                                                        @endforeach
                                                    </td>
                                                    <td>
                                                        {{ $user->created_at->format('j') }}<sup>{{ $user->created_at->format('S') }}</sup>
                                                        {{ $user->created_at->format('M, Y') }}
                                                    </td>
                                                    <td class="d-flex gap-1">
                                                        <button type="button"
                                                            class="btn btn-info btn-sm w-80"
                                                            data-bs-toggle="modal"
                                                            data-bs-target="#userProfileModal"
                                                            data-user-id="{{ $user->id }}"
                                                            data-user-name="{{ $user->name }}"
                                                            data-user-email="{{ $user->email }}"
                                                            data-user-phone="{{ $user->phone_no ?? 'N/A' }}"
                                                            data-user-address="{{ $user->address ?? 'N/A' }}"
                                                            data-user-dob="{{ $user->dob ?? 'N/A' }}"
                                                            data-user-gender="{{ $user->gender ?? 'N/A' }}"
                                                            data-user-role="{{ $user->getRoleNames()->implode(', ') }}"
                                                            data-user-status="{{ $user->status }}"
                                                            data-user-image="{{ $user->image }}"
                                                            data-user-created="{{ $user->created_at->format('M d, Y') }}"
                                                            data-user-company="{{ $user->company_name ?? 'N/A' }}"
                                                            data-user-dot="{{ $user->dot_no ?? 'N/A' }}"
                                                            data-user-mc="{{ $user->mc_no ?? 'N/A' }}">
                                                            Profile
                                                        </button>
                                                        <a href="{{ route('users.edit', $user->id) }}" class="btn btn-warning btn-sm w-80">Edit</a>
                                                        <button type="button"
                                                            class="btn btn-success btn-sm w-80"
                                                            data-bs-toggle="modal"
                                                            data-bs-target="#contactModal"
                                                            data-email="{{ $user->email }}"
                                                            data-phone="{{ $user->phone_no ?? 'N/A' }}">
                                                            Contact
                                                        </button>
                                                    </td>
                                                </tr>
                                            @endforeach
                                        </tbody>
                                    </table>
                                </div>

                            </div>
                        </div>
                    </div> <!-- end card-body -->
                </div> <!-- inner card -->
            </div> <!-- outer card-body -->
        </div> <!-- outer card -->
    </div> <!-- col -->

<script>
    document.addEventListener('DOMContentLoaded', function() {
        // Search functionality for Users by Role table
        const searchInput = document.getElementById('searchUsersByRole');
        const table = document.getElementById('user-approval-table');
        
        if (searchInput && table) {
            searchInput.addEventListener('keyup', function() {
                const filter = this.value.toLowerCase();
                const rows = table.querySelectorAll('tbody tr');
                
                rows.forEach(row => {
                    const cells = row.querySelectorAll('td');
                    let found = false;
                    
                    cells.forEach(cell => {
                        if (cell.textContent.toLowerCase().includes(filter)) {
                            found = true;
                        }
                    });
                    
                    row.style.display = found ? '' : 'none';
                });
            });
        }
    });
</script>
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
                        <input type="email" class="form-control" id="userEmail" readonly>
                    </div>
                    <div class="mb-3">
                        <label for="userContact" class="form-label">Contact</label>
                        <input type="text" class="form-control" id="userContact" readonly>
                    </div>
                </form>
                <!-- <div class="d-flex justify-content-center gap-3">
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
                </div> -->
            </div>
        </div>
    </div>
</div>

<!-- User Profile Modal -->
<div class="modal fade" id="userProfileModal" tabindex="-1" aria-labelledby="userProfileModalLabel" aria-hidden="true">
    <div class="modal-dialog modal-dialog-centered modal-lg">
        <div class="modal-content border border-info">
            <div class="modal-header bg-primary text-white">
                <h5 class="modal-title" id="userProfileModalLabel">User Profile</h5>
                <button type="button" class="btn-close btn-close-white" data-bs-dismiss="modal" aria-label="Close"></button>
            </div>
            <div class="modal-body">
                <div class="row">
                    <!-- Left: Avatar and basic info -->
                    <div class="col-md-4 text-center">
                        <img id="modalUserImage" src="" alt="User Avatar" class="img-fluid rounded-circle mb-3" style="width: 120px; height: 120px; object-fit: cover;">
                        <h5 id="modalUserName" class="mb-1"></h5>
                        <p id="modalUserRole" class="text-muted mb-2"></p>
                        <span id="modalUserStatus" class="badge mb-3"></span>
                        <div class="d-flex gap-2 justify-content-center">
                            <a id="modalEditBtn" href="#" class="btn btn-primary btn-sm">
                                <i class="fa fa-edit"></i> Edit
                            </a>
                        </div>
                    </div>
                    
                    <!-- Right: Detailed info -->
                    <div class="col-md-8">
                        <h6 class="text-primary mb-3">Personal Information</h6>
                        <div class="row g-2 mb-3">
                            <div class="col-6">
                                <label class="text-muted small">Email</label>
                                <p id="modalUserEmail" class="mb-0 fw-medium"></p>
                            </div>
                            <div class="col-6">
                                <label class="text-muted small">Phone</label>
                                <p id="modalUserPhone" class="mb-0 fw-medium"></p>
                            </div>
                            <div class="col-6">
                                <label class="text-muted small">Date of Birth</label>
                                <p id="modalUserDob" class="mb-0 fw-medium"></p>
                            </div>
                            <div class="col-6">
                                <label class="text-muted small">Gender</label>
                                <p id="modalUserGender" class="mb-0 fw-medium"></p>
                            </div>
                            <div class="col-12">
                                <label class="text-muted small">Address</label>
                                <p id="modalUserAddress" class="mb-0 fw-medium"></p>
                            </div>
                        </div>

                        <h6 class="text-primary mb-3 mt-4">Company Information</h6>
                        <div class="row g-2 mb-3">
                            <div class="col-6">
                                <label class="text-muted small">Company Name</label>
                                <p id="modalUserCompany" class="mb-0 fw-medium"></p>
                            </div>
                            <div class="col-6">
                                <label class="text-muted small">DOT Number</label>
                                <p id="modalUserDot" class="mb-0 fw-medium"></p>
                            </div>
                            <div class="col-6">
                                <label class="text-muted small">MC Number</label>
                                <p id="modalUserMc" class="mb-0 fw-medium"></p>
                            </div>
                            <div class="col-6">
                                <label class="text-muted small">Member Since</label>
                                <p id="modalUserCreated" class="mb-0 fw-medium"></p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            <div class="modal-footer">
                <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">Close</button>
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

@push('scripts')
    <script>
        document.addEventListener('DOMContentLoaded', function () {
            const contactModal = document.getElementById('contactModal');

            contactModal.addEventListener('show.bs.modal', function (event) {
                const button = event.relatedTarget;

                const email = button.getAttribute('data-email') || '';
                const phone = button.getAttribute('data-phone') || 'N/A';

                contactModal.querySelector('#userEmail').value = email;
                contactModal.querySelector('#userContact').value = phone;
            });

            // User Profile Modal
            const userProfileModal = document.getElementById('userProfileModal');
            
            userProfileModal.addEventListener('show.bs.modal', function (event) {
                const button = event.relatedTarget;
                
                // Status mapping
                const statusMap = {
                    0: {text: 'Onboarding', class: 'bg-secondary'},
                    1: {text: 'Approved', class: 'bg-success'},
                    2: {text: 'Rejected', class: 'bg-danger'},
                    3: {text: 'Pending Review', class: 'bg-warning'},
                    4: {text: 'Pending OTP', class: 'bg-info'},
                    5: {text: 'Needs Revision', class: 'bg-warning'}
                };
                
                // Get data from button
                const userId = button.getAttribute('data-user-id');
                const userName = button.getAttribute('data-user-name');
                const userEmail = button.getAttribute('data-user-email');
                const userPhone = button.getAttribute('data-user-phone');
                const userAddress = button.getAttribute('data-user-address');
                const userDob = button.getAttribute('data-user-dob');
                const userGender = button.getAttribute('data-user-gender');
                const userRole = button.getAttribute('data-user-role');
                const userStatus = button.getAttribute('data-user-status');
                const userImage = button.getAttribute('data-user-image');
                const userCreated = button.getAttribute('data-user-created');
                const userCompany = button.getAttribute('data-user-company');
                const userDot = button.getAttribute('data-user-dot');
                const userMc = button.getAttribute('data-user-mc');
                
                // Populate modal
                document.getElementById('modalUserName').textContent = userName;
                document.getElementById('modalUserEmail').textContent = userEmail;
                document.getElementById('modalUserPhone').textContent = userPhone;
                document.getElementById('modalUserAddress').textContent = userAddress;
                document.getElementById('modalUserDob').textContent = userDob;
                document.getElementById('modalUserGender').textContent = userGender;
                document.getElementById('modalUserRole').textContent = userRole;
                document.getElementById('modalUserCreated').textContent = userCreated;
                document.getElementById('modalUserCompany').textContent = userCompany;
                document.getElementById('modalUserDot').textContent = userDot;
                document.getElementById('modalUserMc').textContent = userMc;
                
                // Status badge
                const status = statusMap[userStatus] || {text: 'Unknown', class: 'bg-secondary'};
                const statusBadge = document.getElementById('modalUserStatus');
                statusBadge.textContent = status.text;
                statusBadge.className = 'badge mb-3 ' + status.class;
                
                // User image
                const imageUrl = userImage ? 
                    '{{ route("admin.serve-kyc-file") }}?path=' + encodeURIComponent(userImage) : 
                    '{{ asset("assets/images/user/user.png") }}';
                document.getElementById('modalUserImage').src = imageUrl;
                
                // Edit button link
                document.getElementById('modalEditBtn').href = '/users/' + userId + '/edit';
            });
        });
    </script>
@endpush

