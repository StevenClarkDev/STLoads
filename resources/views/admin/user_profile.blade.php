@extends('layout.app')
@section('content')
    <div class="row">
        <div class="col-xl-12 mt-3">
            <div class="card mx-3">
                <div class="card-body">
                    <div class="row gy-4 px-3">
                        <!-- Left Column -->
                        <div class="col-xl-8">
                            <h5 class="mb-3">User Information</h5>
                            <div class="row g-3">
                                <div class="col-sm-6">
                                    <label class="form-label">Name</label>
                                    <input class="form-control" id="userName" type="text" value="{{ $user->name }}" readonly>
                                </div>
                                <div class="col-sm-6">
                                    <label class="form-label">Role</label>
                                    <input class="form-control" type="text"
                                        value="{{ $user->getRoleNames()->implode(', ') }}" readonly>
                                </div>
                                <div class="col-sm-6">
                                    <label class="form-label">Email</label>
                                    <input class="form-control" type="email" value="{{ $user->email }}" readonly>
                                </div>
                                <div class="col-sm-6">
                                    <label class="form-label">DOB</label>
                                    <input class="form-control" type="date" value="{{ $user->dob }}" readonly>
                                </div>
                                <div class="col-sm-6">
                                    <label class="form-label">Gender</label>
                                    <input class="form-control" type="text" value="{{ $user->gender }}" readonly>
                                </div>
                                <div class="col-sm-6">
                                    <label class="form-label">DOJ</label>
                                    <input class="form-control" type="text" value="{{ $user->created_at }}" readonly>
                                </div>
                                <div class="col-12">
                                    <label class="form-label">Address</label>
                                    <input class="form-control" type="text" value="{{ $user->address }}" readonly>
                                </div>
                            </div>
                        </div>

                        <!-- Right Column -->
                        <div class="col-xl-4">
                            <div class="card social-profile mb-0">
                                <div class="card-body bg-primary rounded-4">
                                    <div class="social-img-wrap my-2">
                                        <div class="social-img">
                                            <img class="img-fluid rounded-circle"
                                                src="{{ asset('storage/' . $user->image) }}" alt="profile">
                                        </div>
                                        <div class="edit-icon">
                                            <svg>
                                                <use href="../assets/svg/icon-sprite.svg#profile-check"></use>
                                            </svg>
                                        </div>
                                    </div>
                                    <div class="social-details text-white text-center">
                                        <h5 class="mb-1 text-white">{{ $user->name }}</h5>
                                        <span class="text-light mb-4 d-block">{{ $user->email }}</span>

                                        <ul
                                            class="social-follow list-unstyled d-flex justify-content-between mt-4 mb-2 px-2">
                                            <li class="text-center">
                                                <h6 class="mb-0 pt-2 text-white">{{ $user->id }}</h6>
                                                <span class="text-light small">User ID</span>
                                            </li>
                                            <li class="text-center">
                                                <button class="btn btn-sm btn-link p-0 text-white" data-bs-toggle="modal"
                                                    data-bs-target="#kycModal" title="Download KYC"
                                                    style="width:18px; height:18px;" data-user-id="{{ $user->id }}">
                                                    <i data-feather="download" style="cursor:pointer;"></i>
                                                </button>
                                                <span class="d-block text-light small mt-2">Download KYC</span>
                                            </li>
                                            <li class="text-center">
                                                <button class="btn btn-sm btn-link p-0 text-white" data-bs-toggle="modal"
                                                    data-bs-target="#cnicModal" data-user-id="{{ $user->id }}"
                                                    title="Download CNIC" style="width:18px; height:18px;">
                                                    <i data-feather="download" style="cursor:pointer;"></i>
                                                </button>
                                                <span class="d-block text-light small mt-2">Download CNIC</span>
                                            </li>
                                        </ul>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div> <!-- end row -->
                </div>
            </div>
        </div>
    </div>

    <!-- CNIC Modal -->
    <div class="modal fade" id="cnicModal" tabindex="-1" aria-labelledby="cnicModalLabel" aria-hidden="true">
        <div class="modal-dialog modal-dialog-centered">
            <div class="modal-content border border-primary">
                <div class="modal-header">
                    <h5 class="modal-title">Admin Verification</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <p>Please enter your admin password to download CNIC:</p>
                    <input type="password" id="adminPassword" class="form-control" placeholder="Enter admin password">
                    <div id="passwordError" class="text-danger mt-2 d-none">Incorrect password!</div>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-secondary" onclick="verifyAndDownload()">Confirm</button>
                    <a id="downloadCnicLink" href="#" class="d-none" download></a>
                    <button type="button" class="btn btn-dark" data-bs-dismiss="modal">Cancel</button>
                </div>
            </div>
        </div>
    </div>

    <!-- KYC Modal -->
    <div class="modal fade" id="kycModal" tabindex="-1" aria-labelledby="KYCLabel" aria-hidden="true">
        <div class="modal-dialog modal-dialog-centered">
            <div class="modal-content border border-primary">
                <div class="modal-header">
                    <h5 class="modal-title">Admin Verification</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <p>Please enter your admin password to download User Docs:</p>
                    <input type="password" id="adminPasswordKYC" class="form-control" placeholder="Enter admin password">
                    <div id="passwordErrorKYC" class="text-danger mt-2 d-none">Incorrect password!</div>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-secondary" onclick="verifyAndDownloadKYC()">Confirm</button>
                    <a id="downloadKYCLink" href="#" class="d-none" download></a>
                    <button type="button" class="btn btn-dark" data-bs-dismiss="modal">Cancel</button>
                </div>
            </div>
        </div>
    </div>
@endsection

<script src="{{ url('assets/js/jquery.min.js') }}"></script>
<script src="{{ url('assets/js/bootstrap.bundle.min.js') }}"></script>
<script src="{{ url('assets/js/feather.min.js') }}"></script>
<script src="{{ url('assets/js/script.js') }}"></script>
<script src="https://cdn.jsdelivr.net/npm/sweetalert2@11"></script>

<script>
    document.addEventListener('DOMContentLoaded', function () {
        feather.replace();

        let selectedUserId = null;
        let selectedDocType = null;

        const kycModal = document.getElementById('kycModal');
        kycModal.addEventListener('show.bs.modal', function (event) {
            const buttonKYC = event.relatedTarget;
            selectedUserId = buttonKYC.getAttribute('data-user-id');
        });

        window.verifyAndDownloadKYC = function () {
            const passwordKYC = document.getElementById('adminPasswordKYC').value;

            fetch('/verify-admin-password', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'X-CSRF-TOKEN': '{{ csrf_token() }}'
                },
                body: JSON.stringify({
                    password: passwordKYC
                })
            })
                .then(response => response.json())
                .then(data => {
                    if (data.success) {
                        document.getElementById('passwordErrorKYC').classList.add('d-none');
                        const name = document.getElementById('userName').value;

                        const modalKYC = bootstrap.Modal.getInstance(document.getElementById(
                            'kycModal'));
                        modalKYC.hide();

                        document.getElementById('adminPasswordKYC').value = '';


                        fetch(`/get-user-file/${selectedUserId}`)
                            .then(response => response.json())
                            .then(data => {
                                if (data.files && data.files.length) {
                                    data.files.forEach((file) => {
                                        const link = document.createElement('a');
                                        link.href = file.url;

                                        const docType = file.type;
                                        const extension = file.url.split('.').pop().split(/\#|\?/)[0]; // e.g., jpg, png, pdf
                                        const filename = `${docType}_${name}.${extension}`;

                                        link.download = filename;
                                        document.body.appendChild(link);
                                        link.click();
                                        document.body.removeChild(link);
                                    });
                                } else {
                                    Swal.fire({
                                        icon: 'warning',
                                        title: 'No Files Found',
                                        text: 'The requested documents are not available.',
                                    });
                                }
                            });

                    } else {
                        document.getElementById('passwordErrorKYC').classList.remove('d-none');
                    }
                });
        };
        const cnicModal = document.getElementById('cnicModal');
        cnicModal.addEventListener('show.bs.modal', function (event) {
            const button = event.relatedTarget;
            selectedUserId = button.getAttribute('data-user-id');
        });

        window.verifyAndDownload = function () {
            const password = document.getElementById('adminPassword').value;

            fetch('/verify-admin-password', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'X-CSRF-TOKEN': '{{ csrf_token() }}'
                },
                body: JSON.stringify({
                    password
                })
            })
                .then(response => response.json())
                .then(data => {
                    if (data.success) {
                        document.getElementById('passwordError').classList.add('d-none');
                        const name = document.getElementById('userName').value;

                        const modal = bootstrap.Modal.getInstance(document.getElementById('cnicModal'));
                        modal.hide();

                        document.getElementById('adminPassword').value = '';


                        fetch(`/get-cnic-file/${selectedUserId}`)
                            .then(response => response.json())
                            .then(data => {
                                if (data.files && data.files.length) {
                                    data.files.forEach((file) => {
                                        const link = document.createElement('a');
                                        link.href = file.url;

                                        const docType = file.type;
                                        const extension = file.url.split('.').pop().split(/\#|\?/)[0]; // e.g., jpg, png, pdf
                                        const filename = `${docType}_${name}.${extension}`;

                                        link.download = filename;
                                        document.body.appendChild(link);
                                        link.click();
                                        document.body.removeChild(link);
                                    });
                                } else {
                                    Swal.fire({
                                        icon: 'warning',
                                        title: 'No CNIC Files Found',
                                        text: 'The requested CNIC front/back documents are not available.',
                                    });
                                }
                            });

                    } else {
                        document.getElementById('passwordError').classList.remove('d-none');
                    }
                });
        };
    });
</script>