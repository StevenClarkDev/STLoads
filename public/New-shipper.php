<?php include 'header.php'; ?>

<div class="page-body">
<div class="container-fluid">

<div class="page-title">
<h3>Account Registration</h3>
</div>

<div class="row">
<div class="col-sm-12">

<div class="card">
<div class="card-body">

<style>

.step{display:none;}
.step.active{display:block;}

#progressbar{
display:flex;
gap:10px;
margin-bottom:25px;
}

#progressbar span{
padding:6px 14px;
border-radius:6px;
background:#f1f1f1;
font-size:14px;
}

#progressbar span.active{
background:#1f4f75;
color:#fff;
}

</style>

<ul id="progressbar">
<li><span class="active">1. Account</span></li>
<li><span>2. Personal</span></li>
<li><span>3. Company</span></li>
<li><span>4. Documents</span></li>
<li><span>5. Compliance</span></li>
</ul>

<form id="kycForm">

<!-- STEP 1 -->
<div class="step active">

<h5>Account Information</h5>

<div class="row">

<div class="col-md-6 mb-3">
<label>Email Address</label>
<input type="email" class="form-control" required>
</div>

<div class="col-md-6 mb-3">
<label>Phone Number</label>
<input type="text" class="form-control">
</div>

<div class="col-md-6 mb-3">
<label>Password</label>
<input type="password" class="form-control">
</div>

<div class="col-md-6 mb-3">
<label>Confirm Password</label>
<input type="password" class="form-control">
</div>

</div>
</div>

<!-- STEP 2 -->
<div class="step">

<h5>Personal Information</h5>

<div class="row">

<div class="col-md-6 mb-3">
<label>Full Legal Name</label>
<input type="text" class="form-control">
</div>

<div class="col-md-6 mb-3">
<label>Date of Birth</label>
<input type="date" class="form-control">
</div>

<div class="col-md-6 mb-3">
<label>Gender</label>
<select class="form-control">
<option>Select</option>
<option>Male</option>
<option>Female</option>
<option>Other</option>
</select>
</div>

<div class="col-md-6 mb-3">
<label>Nationality</label>
<input type="text" class="form-control">
</div>

</div>
</div>

<!-- STEP 3 -->
<div class="step">

<h5>Company Information</h5>

<div class="row">

<div class="col-md-6 mb-3">
<label>Company Name</label>
<input type="text" class="form-control">
</div>

<div class="col-md-6 mb-3">
<label>Registration Number</label>
<input type="text" class="form-control">
</div>

<div class="col-md-6 mb-3">
<label>Tax ID</label>
<input type="text" class="form-control">
</div>

<div class="col-md-6 mb-3">
<label>Country of Incorporation</label>
<input type="text" class="form-control">
</div>

</div>
</div>

<!-- STEP 4 -->
<div class="step">

<h5>Upload Documents</h5>

<div class="row">

<div class="col-md-6 mb-3">
<label>Government ID</label>
<input type="file" class="form-control">
</div>

<div class="col-md-6 mb-3">
<label>Selfie / Facial Verification</label>
<input type="file" class="form-control">
</div>

<div class="col-md-6 mb-3">
<label>Certificate of Incorporation</label>
<input type="file" class="form-control">
</div>

<div class="col-md-6 mb-3">
<label>Tax Registration Certificate</label>
<input type="file" class="form-control">
</div>

</div>
</div>

<!-- STEP 5 -->
<div class="step">

<h5>Compliance</h5>

<div class="form-check mb-2">
<input class="form-check-input" type="checkbox">
<label class="form-check-label">
Consent to sanctions screening
</label>
</div>

<div class="form-check mb-2">
<input class="form-check-input" type="checkbox">
<label class="form-check-label">
Politically Exposed Person declaration
</label>
</div>

<div class="mb-3">
<label>Source of Funds</label>
<textarea class="form-control"></textarea>
</div>

<div class="form-check">
<input class="form-check-input" type="checkbox">
<label class="form-check-label">
Agree to AML policies
</label>
</div>

</div>

<div class="d-flex justify-content-between mt-4">

<button type="button" class="btn btn-secondary" id="prevBtn">
Previous
</button>

<button type="button" class="btn btn-primary" id="nextBtn">
Next
</button>

</div>

</form>

</div>
</div>

</div>
</div>
</div>
</div>

<?php include 'footer.php'; ?>