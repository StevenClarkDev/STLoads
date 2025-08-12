@extends('auth.app')
@section('content')
    <div class="card p-5 rounded shadow my-4" style="max-width: 1100px; width: 100%;">
        <div class="text-center mb-4">
            <img src="../assets/images/stloads/logo-bg_none-small.png" alt="Load Board Logo" style="max-width: 30%;">
        </div>

        <div class="numbering-wizard mb-4">
            <div class="d-flex justify-content-between position-relative">
                <div class="step active" data-step="1">1</div>
                <div class="step" data-step="2">2</div>
                <div class="step" data-step="3">3</div>
                <div class="step" data-step="4">4</div>
                <div class="progress-bar position-absolute"></div>
            </div>
            <div class="d-flex justify-content-between mt-2">
                <div class="step-label active" data-step="1">Basic Info</div>
                <div class="step-label" data-step="2">Cart Info</div>
                <div class="step-label" data-step="3">Feedback</div>
                <div class="step-label" data-step="4">Finish</div>
            </div>
        </div>

        <form class="row g-3" action="{{ route('register') }}" method="POST" enctype="multipart/form-data"
            id="registrationForm">
            @csrf

            <!-- All step contents wrapped in a scrollable container -->
            <div class="step-container" style="height: 300px; overflow-y: auto;">
                <!-- Step 1: Basic Info -->
                <div class="step-content active" data-step="1">
                    <div class="my-3" style="border-bottom: 1px solid #ecf3fa;">
                        <h4 class="text-center">Basic Information</h4>
                        <p class="text-muted text-center">Enter your basic details to get started</p>
                    </div>
                    <div class="col-md-12 d-flex gap-2">
                        <div class="col-md-6 position-relative">
                            <label>Email*</label>
                            <div class="input-group">
                                <input id="email" class="form-control pe-5 rounded-2" type="email" name="email"
                                    placeholder="you@example.com" required>
                                <i id="email-icon"
                                    class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                            </div>
                        </div>

                        <div class="col-md-6 position-relative">
                            <label>First Name*</label>
                            <div class="input-group">
                                <input id="name" class="form-control pe-5 rounded-2" type="text" name="name"
                                    placeholder="Enter your name" required>
                                <i id="name-icon"
                                    class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                            </div>
                        </div>
                    </div>

                    <div class="col-md-12 d-flex gap-2 mt-3">
                        <div class="col-md-6 position-relative">
                            <label>Password*</label>
                            <div class="input-group">
                                <input id="password" class="form-control pe-5 rounded-2" type="password" name="password"
                                    placeholder="Enter password" required>
                                <i id="password-icon"
                                    class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                            </div>
                            <small id="password-error" class="text-danger d-none">Password must be at least 8 characters and
                                contain a letter and number.</small>
                        </div>

                        <div class="col-md-6 position-relative">
                            <label>Confirm Password*</label>
                            <div class="input-group">
                                <input id="confirm-password" class="form-control pe-5 rounded-2" type="password"
                                    name="password_confirmation" placeholder="Enter confirm password" required>
                                <i id="confirm-password-icon"
                                    class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Step 2: Additional Info -->
                <div class="step-content" data-step="2">
                    <div class="my-3" style="border-bottom: 1px solid #ecf3fa;">
                        <h4 class="text-center">Additional Information</h4>
                        <p class="text-muted text-center">Provide some more details about yourself</p>
                    </div>

                    <div class="col-md-12 d-flex gap-2">
                        <div class="flex-fill position-relative">
                            <label>Gender</label>
                            <select id="gender" class="form-control pe-5 rounded-2 text-muted" name="gender" required>
                                <option disabled selected>Select Gender</option>
                                <option value="Male">Male</option>
                                <option value="Female">Female</option>
                                <option value="Other">Other</option>
                            </select>
                        </div>
                        <div class="flex-fill position-relative">
                            <label>Role</label>
                            <input class="form-control" type="text" name="role" value="{{ $role_name }}" readonly>
                            <input type="hidden" name="role_id" value="{{ $id }}">
                        </div>
                        <div class="flex-fill position-relative">
                            <label>Date of Birth</label>
                            <input class="form-control pe-5 rounded-2" type="date" name="dob" required>
                        </div>
                    </div>

                    <div class="col-md-12 d-flex gap-2 mt-3">
                        <div class="col-md-8">
                            <label>Address</label>
                            <input class="form-control" type="text" name="address" placeholder="Complete Address" required>
                        </div>

                        <div class="col-md-4 position-relative">
                            <label>CNIC #</label>
                            <input class="form-control" type="text" name="cnic_no" placeholder="00000-0000000-0" required>
                        </div>
                    </div>
                </div>

                <!-- Step 3: Profile Upload -->
                <div class="step-content" data-step="3">
                    <div class="my-3" style="border-bottom: 1px solid #ecf3fa;">
                        <h4 class="text-center">Profile Information</h4>
                        <p class="text-muted text-center">Upload your profile picture and agree to terms</p>
                    </div>
                    <div class="col-md-12">
                        <label>Profile Upload</label>
                        <div class="d-flex align-items-center border rounded-2 p-1">
                            <button type="button" class="btn d-flex align-items-center px-2"
                                onclick="document.getElementById('user_image').click()">
                                <i class="fas fa-upload"></i>
                            </button>
                            <span id="user_image_name" class="ms-3 text-muted small">No file chosen</span>
                            <input type="file" id="user_image" name="user_image" accept="image/*" class="d-none" required>
                        </div>
                    </div>

                    <div class="col-12 mt-3">
                        <div class="form-check">
                            <input class="form-check-input" type="checkbox" id="terms" name="terms" required>
                            <label class="form-check-label" for="terms">
                                Agree to terms and conditions
                            </label>
                        </div>
                    </div>
                </div>

                <!-- Step 4: Review and Submit -->
                <div class="step-content" data-step="4">
                    <div class="text-center mb-4">
                        <h4 class="mb-2">Review Your Information</h4>
                        <p class="text-muted">Please verify all details before submitting your registration</p>
                    </div>

                    <div class="row g-2">
                        <!-- Basic Information Card -->
                        <div class="col-md-6">
                            <div class="card border-light review-card h-80">
                                <div class="card-header bg-primary text-white py-3">
                                    <h6 class="mb-0"><i class="fas fa-user-circle me-2"></i>Basic Information</h6>
                                </div>
                                <div class="card-body">
                                    <div class="d-flex mb-3">
                                        <div class="flex-shrink-0 text-secondary mx-2">
                                            <i class="fas fa-envelope"></i>
                                        </div>
                                        <div>
                                            <div id="review-email"></div>
                                        </div>
                                    </div>
                                    <div class="d-flex">
                                        <div class="flex-shrink-0 text-secondary mx-2">
                                            <i class="fas fa-user"></i>
                                        </div>
                                        <div>
                                            <div id="review-name"></div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <!-- Additional Information Card -->
                        <div class="col-md-6">
                            <div class="card border-light review-card h-80">
                                <div class="card-header bg-primary text-white py-3">
                                    <h6 class="mb-0"><i class="fas fa-info-circle me-2"></i>Additional Information</h6>
                                </div>
                                <div class="card-body">
                                    <div class="col-md-12 d-flex flex-row">
                                        <div class="d-flex align-items-center mb-3 flex-fill">
                                            <div class="flex-shrink-0 text-secondary mx-2">
                                                <i class="fas fa-birthday-cake"></i>
                                            </div>
                                            <div>
                                                <div id="review-dob"></div>
                                            </div>
                                        </div>
                                        <div class="d-flex align-items-center mb-3 flex-fill">
                                            <div class="flex-shrink-0 text-secondary mx-2">
                                                <i class="fas fa-venus-mars"></i>
                                            </div>
                                            <div>
                                                <div id="review-gender"></div>
                                            </div>
                                        </div>
                                    </div>
                                    <div class="d-flex">
                                        <div class="flex-shrink-0 text-secondary mx-2">
                                            <i class="fas fa-map-marker-alt"></i>
                                        </div>
                                        <div>
                                            <div id="review-address"></div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <!-- Account Information Card -->
                        <div class="col-12">
                            <div class="card border-light review-card">
                                <div class="card-header bg-primary text-white py-3">
                                    <h6 class="mb-0"><i class="fas fa-id-card me-2"></i>Account Information</h6>
                                </div>
                                <div class="card-body">
                                    <div class="d-flex">
                                        <div class="flex-shrink-0 text-secondary mx-2">
                                            <i class="fas fa-user-tag"></i>
                                        </div>
                                        <div>
                                            <div id="review-role"></div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Navigation Buttons -->
            <div class="col-12 d-flex justify-content-between mt-4">
                <button type="button" class="btn btn-outline-secondary" id="prevBtn" disabled>Back</button>
                <button type="button" class="btn btn-primary" id="nextBtn">Next</button>
                <button type="submit" class="btn btn-success" id="submitBtn" style="display: none;">Submit</button>
            </div>

            <!-- Laravel error/success messages -->
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

    <style>
        .numbering-wizard {
            margin-bottom: 2rem;
        }

        .step {
            width: 40px;
            height: 40px;
            border-radius: 50%;
            background-color: #e9ecef;
            display: flex;
            align-items: center;
            justify-content: center;
            font-weight: bold;
            z-index: 2;
        }

        .step.active {
            background-color: #1F537B;
            color: white;
        }

        .step-label {
            font-size: 0.9rem;
            color: #6c757d;
        }

        .step-label.active {
            font-weight: bold;
            color: #1F537B;
        }

        .progress-bar {
            height: 2px;
            background-color: #e9ecef;
            top: 20px;
            left: 20px;
            right: 20px;
            z-index: 1;
        }

        .step-content {
            display: none;
        }

        .step-content.active {
            display: block;
        }

        .review-card {
            transition: all 0.3s ease;
            border-width: 1px;
            border-color: #dee2e6 !important;
            box-shadow: 0 2px 10px rgba(0, 0, 0, 0.05);
        }

        .review-card:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
        }

        .card-header {
            border-bottom: 1px solid #dee2e6;
            text-align: center;
            /* background-color: #f8f9fa !important; */
        }

        .step-container {
            scrollbar-width: thin;
            scrollbar-color: #1F537B #f1f1f1;
        }

        .step-container::-webkit-scrollbar {
            width: 6px;
        }

        .step-container::-webkit-scrollbar-track {
            background: #f1f1f1;
        }

        .step-container::-webkit-scrollbar-thumb {
            background-color: #1F537B;
            border-radius: 6px;
        }

        .fa-check-circle.text-success {
            color: #28a745 !important;
        }

        .border-light {
            border-color: #ecf3fa !important;
        }
    </style>

    <script>
        document.addEventListener('DOMContentLoaded', function () {
            const steps = document.querySelectorAll('.step-content');
            const stepIndicators = document.querySelectorAll('.step');
            const stepLabels = document.querySelectorAll('.step-label');
            const prevBtn = document.getElementById('prevBtn');
            const nextBtn = document.getElementById('nextBtn');
            const submitBtn = document.getElementById('submitBtn');
            let currentStep = 1;

            // Initialize progress bar
            updateProgressBar();

            // Validation check for input fields
            function validateInput(input, iconId) {
                const icon = document.getElementById(iconId);
                if (input.value.trim() !== '') {
                    icon.classList.remove('text-muted');
                    icon.classList.add('text-success');
                } else {
                    icon.classList.remove('text-success');
                    icon.classList.add('text-muted');
                }
            }

            // Add event listeners for input validation
            document.getElementById('email').addEventListener('input', function () {
                validateInput(this, 'email-icon');
            });
            document.getElementById('name').addEventListener('input', function () {
                validateInput(this, 'name-icon');
            });
            document.getElementById('password').addEventListener('input', function () {
                validateInput(this, 'password-icon');
            });
            document.getElementById('confirm-password').addEventListener('input', function () {
                validateInput(this, 'confirm-password-icon');
            });

            // Next button click handler
            nextBtn.addEventListener('click', function () {
                if (validateStep(currentStep)) {
                    if (currentStep < steps.length) {
                        // Move to next step
                        document.querySelector(`.step-content[data-step="${currentStep}"]`).classList.remove('active');
                        document.querySelector(`.step[data-step="${currentStep}"]`).classList.remove('active');
                        document.querySelector(`.step-label[data-step="${currentStep}"]`).classList.remove('active');

                        currentStep++;

                        document.querySelector(`.step-content[data-step="${currentStep}"]`).classList.add('active');
                        document.querySelector(`.step[data-step="${currentStep}"]`).classList.add('active');
                        document.querySelector(`.step-label[data-step="${currentStep}"]`).classList.add('active');

                        updateButtons();
                        updateProgressBar();

                        // If we're on the last step, update the review section
                        if (currentStep === 4) {
                            updateReviewSection();
                        }
                    }
                }
            });

            // Previous button click handler
            prevBtn.addEventListener('click', function () {
                if (currentStep > 1) {
                    document.querySelector(`.step-content[data-step="${currentStep}"]`).classList.remove('active');
                    document.querySelector(`.step[data-step="${currentStep}"]`).classList.remove('active');
                    document.querySelector(`.step-label[data-step="${currentStep}"]`).classList.remove('active');

                    currentStep--;

                    document.querySelector(`.step-content[data-step="${currentStep}"]`).classList.add('active');
                    document.querySelector(`.step[data-step="${currentStep}"]`).classList.add('active');
                    document.querySelector(`.step-label[data-step="${currentStep}"]`).classList.add('active');

                    updateButtons();
                    updateProgressBar();
                }
            });

            // Update button states based on current step
            function updateButtons() {
                prevBtn.disabled = currentStep === 1;
                nextBtn.style.display = currentStep === steps.length ? 'none' : 'block';
                submitBtn.style.display = currentStep === steps.length ? 'block' : 'none';
            }

            // Update progress bar
            function updateProgressBar() {
                const progressBar = document.querySelector('.progress-bar');
                const progressPercentage = ((currentStep - 1) / (steps.length - 1)) * 100;
                progressBar.style.background = `linear-gradient(to right, #1F537B ${progressPercentage}%, #e9ecef ${progressPercentage}%)`;
            }

            // Validate current step before proceeding
            function validateStep(step) {
                let isValid = true;

                if (step === 1) {
                    const email = document.getElementById('email').value;
                    const name = document.getElementById('name').value;
                    const password = document.getElementById('password').value;
                    const confirmPassword = document.getElementById('confirm-password').value;

                    if (!email || !name || !password || !confirmPassword) {
                        alert('Please fill all required fields');
                        isValid = false;
                    } else if (password !== confirmPassword) {
                        alert('Passwords do not match');
                        isValid = false;
                    }
                } else if (step === 2) {
                    const dob = document.querySelector('input[name="dob"]').value;
                    const gender = document.getElementById('gender').value;
                    const address = document.querySelector('input[name="address"]').value;
                    const cnic = document.querySelector('input[name="cnic_no"]').value;

                    if (!dob || gender === 'Select Gender' || !address || !cnic) {
                        alert('Please fill all required fields');
                        isValid = false;
                    }
                } else if (step === 3) {
                    const fileInput = document.getElementById('user_image');
                    const terms = document.getElementById('terms');

                    if (!fileInput.files.length) {
                        alert('Please upload a profile picture');
                        isValid = false;
                    } else if (!terms.checked) {
                        alert('You must agree to the terms and conditions');
                        isValid = false;
                    }
                }

                return isValid;
            }

            function updateReviewSection() {
                document.getElementById('review-email').innerHTML = `<strong>Email:</strong> ${document.getElementById('email').value}`;
                document.getElementById('review-name').innerHTML = `<strong>Name:</strong> ${document.getElementById('name').value}`;
                document.getElementById('review-dob').innerHTML = `<strong>Date of Birth:</strong> ${document.querySelector('input[name="dob"]').value}`;
                document.getElementById('review-gender').innerHTML = `<strong>Gender:</strong> ${document.getElementById('gender').value}`;
                document.getElementById('review-address').innerHTML = `<strong>Address:</strong> ${document.querySelector('input[name="address"]').value}`;
                document.getElementById('review-role').innerHTML = `<strong>Role:</strong> ${document.querySelector('input[name="role"]').value}`;
            }

            // File upload name display
            document.getElementById('user_image').addEventListener('change', function () {
                const fileName = this.files[0]?.name || 'No file chosen';
                document.getElementById('user_image_name').textContent = fileName;
            });

            // Initialize validation icons
            validateInput(document.getElementById('email'), 'email-icon');
            validateInput(document.getElementById('name'), 'name-icon');
            validateInput(document.getElementById('password'), 'password-icon');
            validateInput(document.getElementById('confirm-password'), 'confirm-password-icon');
        });
    </script>
@endsection