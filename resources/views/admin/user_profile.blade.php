@extends('layout.app')

@section('content')
            <div class="col-xl-9 box-col-6 p-3">
                <div class="card mx-4">
                    <div class="card-body">
                        <div class="row gy-4 px-4">
                            <!-- Left Column -->
                            <div class="col-xl-8">
                                <h5 class="mb-3">User Information</h5>
                                <div class="row g-3">
                                    <div class="col-sm-6">
                                        <label class="form-label">Name</label>
                                        <input class="form-control" type="text" value="{{ $user->name }}" readonly>
                                    </div>
                                    {{-- <div class="col-sm-6">
                                        <label class="form-label">Last Name</label>
                                        <input class="form-control" type="text" value="Doe" readonly>
                                    </div> --}}
                                    <div class="col-sm-6">
                                        <label class="form-label">Role</label>
                                        <input class="form-control" type="text" value="{{ $user->getRoleNames()->implode(', ') }}" readonly>
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
                                    <div class="col-12">
                                        <label class="form-label">Address</label>
                                        <input class="form-control" type="text"
                                            value="{{ $user->address }}" readonly>
                                    </div>
                                </div>
                            </div>
                            <div class="col-xl-4">
                                <div class="card social-profile">
                                    <div class="card-body bg-secondary rounded-4">
                                        <div class="social-img-wrap my-2">
                                            <div class="social-img"><img class="img-fluid rounded-circle"
                                                    src="../assets/images/user/2.png" alt="profile"></div>
                                            <div class="edit-icon">
                                                <svg>
                                                    <use href="../assets/svg/icon-sprite.svg#profile-check"></use>
                                                </svg>
                                            </div>
                                        </div>
                                        <div class="social-details text-white">
                                            <h5 class="mb-1 text-white">
                                                {{ $user->name }}
                                            </h5>
                                            <span class="text-light mb-4">{{ $user->email }}</span>

                                            <ul
                                                class="social-follow list-unstyled d-flex justify-content-between mt-4 mb-2 p-2">
                                                <li class="text-center">
                                                    <h6 class="mb-0 text-white mb-2">C-0223</h6>
                                                    <span class="text-light small">User ID</span>
                                                </li>
                                                <li class="text-center">
                                                    <h6 class="mb-0 text-white mb-2">12 July 2023</h6>
                                                    <span class="text-light small">Joining Date</span>
                                                </li>
                                                <li class="text-center">
                                                    <button class="btn btn-sm btn-link p-0 text-white"
                                                        data-bs-toggle="modal" data-bs-target="#cnicModal"
                                                        title="Download CNIC" style="width:18px; height:18px;"></button>
                                                    <i data-feather="download" style="cursor:pointer;"
                                                        data-bs-toggle="modal" data-bs-target="#cnicModal"></i>
                                                    </button>
                                                    <span class="d-block text-light small">Download CNIC</span>
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

            <div class="col-md-12">
                <div class="card px-4 mx-4">
                    <div class="card-header">
                        <h5 class="mb-2">Projects History</h5>
                        <div class="card-options"><a class="card-options-collapse" href="#"
                                data-bs-toggle="card-collapse"><i class="fe fe-chevron-up"></i></a><a
                                class="card-options-remove" href="#" data-bs-toggle="card-remove"><i
                                    class="fe fe-x"></i></a></div>
                    </div>
                    <div class="table-responsive add-project rounded-4 px-4">
                        <table class="table card-table table-vcenter text-nowrap">
                            <thead>
                                <tr>
                                    <th>Project Name</th>
                                    <th>Date</th>
                                    <th>Status</th>
                                    <th>Price</th>
                                    <th class="text-center">Action</th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr>
                                    <td><a class="text-inherit" href="#">Untrammelled prevents </a></td>
                                    <td>28 May 2018</td>
                                    <td><span class="status-icon bg-success"></span> Completed</td>
                                    <td>$56,908</td>
                                    <td class="text-center"><a class="icon" href="javascript:void(0)"></a><a
                                            class="btn btn-primary btn-sm" href="javascript:void(0)"><i
                                                class="fa fa-pencil"></i> Edit</a>
                                        <a class="icon" href="javascript:void(0)"></a><a class="icon"
                                            href="javascript:void(0)"></a><a class="btn btn-danger btn-sm"
                                            href="javascript:void(0)"><i class="fa fa-trash"></i>
                                            Delete</a>
                                    </td>
                                </tr>
                                <tr>
                                    <td><a class="text-inherit" href="#">Untrammelled prevents</a></td>
                                    <td>12 June 2018</td>
                                    <td><span class="status-icon bg-danger"></span> On going</td>
                                    <td>$45,087</td>
                                    <td class="text-center"><a class="icon" href="javascript:void(0)"></a><a
                                            class="btn btn-primary btn-sm" href="javascript:void(0)"><i
                                                class="fa fa-pencil"></i> Edit</a>
                                        <a class="icon" href="javascript:void(0)"></a><a class="icon"
                                            href="javascript:void(0)"></a><a class="btn btn-danger btn-sm"
                                            href="javascript:void(0)"><i class="fa fa-trash"></i>
                                            Delete</a>
                                    </td>
                                </tr>
                                <tr>
                                    <td><a class="text-inherit" href="#">Untrammelled prevents</a></td>
                                    <td>12 July 2018</td>
                                    <td><span class="status-icon bg-warning"></span> Pending</td>
                                    <td>$60,123</td>
                                    <td class="text-center"><a class="icon" href="javascript:void(0)"></a><a
                                            class="btn btn-primary btn-sm" href="javascript:void(0)"><i
                                                class="fa fa-pencil"></i> Edit</a>
                                        <a class="icon" href="javascript:void(0)"></a><a class="icon"
                                            href="javascript:void(0)"></a><a class="btn btn-danger btn-sm"
                                            href="javascript:void(0)"><i class="fa fa-trash"></i>
                                            Delete</a>
                                    </td>
                                </tr>
                                <tr>
                                    <td><a class="text-inherit" href="#">Untrammelled prevents</a></td>
                                    <td>14 June 2018</td>
                                    <td><span class="status-icon bg-warning"></span> Pending</td>
                                    <td>$70,435</td>
                                    <td class="text-center"><a class="icon" href="javascript:void(0)"></a><a
                                            class="btn btn-primary btn-sm" href="javascript:void(0)"><i
                                                class="fa fa-pencil"></i> Edit</a>
                                        <a class="icon" href="javascript:void(0)"></a><a class="icon"
                                            href="javascript:void(0)"></a><a class="btn btn-danger btn-sm"
                                            href="javascript:void(0)"><i class="fa fa-trash"></i>
                                            Delete</a>
                                    </td>
                                </tr>
                                <tr>
                                    <td><a class="text-inherit" href="#">Untrammelled prevents</a></td>
                                    <td>25 June 2018</td>
                                    <td><span class="status-icon bg-success"></span> Completed</td>
                                    <td>$15,987</td>
                                    <td class="text-center"><a class="icon" href="javascript:void(0)"></a><a
                                            class="btn btn-primary btn-sm" href="javascript:void(0)"><i
                                                class="fa fa-pencil"></i> Edit</a>
                                        <a class="icon" href="javascript:void(0)"></a><a class="icon"
                                            href="javascript:void(0)"></a><a class="btn btn-danger btn-sm"
                                            href="javascript:void(0)"><i class="fa fa-trash"></i>
                                            Delete</a>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
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
                        <button type="button" class="btn btn-dark" data-bs-dismiss="modal">Cancel</button>
                    </div>

                </div>
            </div>
        </div>

    </div>
    <!-- Hidden download link -->
    <a id="downloadCnicLink" href="{{ asset('dummy_files/cnic.pdf') }}" download style="display:none;"></a>

@endsection

@push('scripts')
    <script>
        feather.replace();

        function verifyAndDownload() {
            const password = document.getElementById('adminPassword').value;

            // Simulated password check — for frontend only
            if (password === 'admin123') {
                document.getElementById('passwordError').classList.add('d-none');

                // Close modal
                const modal = bootstrap.Modal.getInstance(document.getElementById('cnicModal'));
                modal.hide();

                // Simulate CNIC download
                document.getElementById('downloadCnicLink').click();
            } else {
                document.getElementById('passwordError').classList.remove('d-none');
            }
        }
    </script>
@endpush
