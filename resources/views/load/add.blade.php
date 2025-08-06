@extends('layout.app')
@section('content')
    <div class="container-fluid">
        <div class="page-title">
            <div class="row">
                <div class="col-6">
                    <h4>Add Load</h4>
                </div>
                <div class="col-6">
                    <ol class="breadcrumb">
                        <li class="breadcrumb-item"><a href="index.html">
                                <svg class="stroke-icon">
                                    <use href="../assets/svg/icon-sprite.svg#stroke-home"></use>
                                </svg></a></li>
                        <li class="breadcrumb-item"> Add Load</li>
                        <li class="breadcrumb-item active"> Load Details</li>
                    </ol>
                </div>
            </div>
        </div>
    </div>
    <!-- Container-fluid starts-->
    <div class="container-fluid">
        <div class="row">
            <!-- <div class="col-sm-12">
                                                                                    <div class="card">
                                                                                        <div class="card-header">
                                                                                            <h4>Tooltip form validation</h4>
                                                                                            <p class="f-m-light mt-1">
                                                                                                If your form layout allows it, you can swap the <code>.{valid|invalid}</code>-feedback
                                                                                                classes for<code>.{valid|invalid}</code>-tooltip classes to display validation feedback in a
                                                                                                styled tooltip. Be sure to have a parent with <code>position: relative</code> on it for
                                                                                                tooltip positioning.</p>
                                                                                        </div>
                                                                                        <div class="card-body">
                                                                                            <form class="row g-3 needs-validation custom-input" novalidate="">
                                                                                                <div class="col-md-4 position-relative">
                                                                                                    <label class="form-label" for="validationTooltip01">First name</label>
                                                                                                    <input class="form-control" id="validationTooltip01" type="text" placeholder="Mark"
                                                                                                        required="">
                                                                                                    <div class="valid-tooltip">Looks good!</div>
                                                                                                </div>
                                                                                                <div class="col-md-4 position-relative">
                                                                                                    <label class="form-label" for="validationTooltip02">Last name</label>
                                                                                                    <input class="form-control" id="validationTooltip02" type="text" placeholder="Otto"
                                                                                                        required="">
                                                                                                    <div class="valid-tooltip">Looks good!</div>
                                                                                                </div>
                                                                                                <div class="col-md-4 position-relative">
                                                                                                    <label class="form-label" for="validationTooltipUsername">Username</label>
                                                                                                    <div class="input-group has-validation"><span class="input-group-text"
                                                                                                            id="validationTooltipUsernamePrepend">@</span>
                                                                                                        <input class="form-control" id="validationTooltipUsername" type="text"
                                                                                                            aria-describedby="validationTooltipUsernamePrepend" required="">
                                                                                                        <div class="invalid-tooltip">Please choose a unique and valid username.</div>
                                                                                                    </div>
                                                                                                </div>
                                                                                                <div class="col-md-6 position-relative">
                                                                                                    <label class="form-label" for="validationTooltip03">City</label>
                                                                                                    <input class="form-control" id="validationTooltip03" type="text" required="">
                                                                                                    <div class="invalid-tooltip">Please provide a valid city.</div>
                                                                                                </div>
                                                                                                <div class="col-md-3 position-relative">
                                                                                                    <label class="form-label" for="validationTooltip04">State</label>
                                                                                                    <select class="form-select" id="validationTooltip04" required="">
                                                                                                        <option selected="" disabled="" value="">Choose...</option>
                                                                                                        <option>U.S </option>
                                                                                                        <option>Thailand </option>
                                                                                                        <option>India </option>
                                                                                                        <option>U.K</option>
                                                                                                    </select>
                                                                                                    <div class="invalid-tooltip">Please select a valid state.</div>
                                                                                                </div>
                                                                                                <div class="col-md-3 position-relative">
                                                                                                    <label class="form-label" for="validationTooltip05">Zip</label>
                                                                                                    <input class="form-control" id="validationTooltip05" type="text" required="">
                                                                                                    <div class="invalid-tooltip">Please provide a valid zip.</div>
                                                                                                </div>
                                                                                                <div class="col-12">
                                                                                                    <button class="btn btn-primary" type="submit">Submit form</button>
                                                                                                </div>
                                                                                            </form>
                                                                                        </div>
                                                                                    </div>
                                                                                </div> -->
            <div class="col-xl-8">
                <div class="card height-equal">
                    <div class="card-header">
                        <h4>Load Details</h4>
                    </div>
                    <div class="card-body">
                        <form class="row g-3 needs-validation custom-input" novalidate="">
                            <div class="col-md-6">
                                <label class="form-label" for="validationCustom03">Pickup Location</label>
                                <input class="form-control" id="validationCustom03" type="text" required="">
                                <div class="invalid-feedback">Please provide a valid location.</div>
                                <div class="valid-feedback">
                                    Looks's Good!</div>
                            </div>
                            <div class="col-md-6">
                                <label class="form-label" for="validationCustom05">Dropoff Location</label>
                                <input class="form-control" id="validationCustom05" type="text" required="">
                                <div class="invalid-feedback">Please provide a valid location.</div>
                                <div class="valid-feedback">
                                    Looks's Good!</div>
                            </div>
                            <div class="col-md-6">
                                <label class="form-label" for="validationCustom04">Lane</label>
                                <select class="form-select" id="validationCustom04" required="">
                                    <option selected="" disabled="" value="">Choose...</option>
                                    <option>U.K </option>
                                    <option>India </option>
                                    <option>Thailand</option>
                                    <option>Newyork</option>
                                </select>
                                <div class="invalid-feedback">Please select a valid lane.</div>
                                <div class="valid-feedback">
                                    Looks's Good! </div>
                            </div>
                            <div class="col-md-6">
                                <label class="form-label" for="validationCustom04">Mode</label>
                                <select class="form-select" id="validationCustom04" required="">
                                    <option selected="" disabled="" value="">Choose...</option>
                                    <option>By Air </option>
                                    <option>By Ship </option>
                                    <option>By Train</option>
                                    <option>By Vehicle</option>
                                </select>
                                <div class="invalid-feedback">Please select a valid mode.</div>
                                <div class="valid-feedback">
                                    Looks's Good! </div>
                            </div>
                            <div class="col-12">
                                <label class="form-label" for="validationCustom01">Client name</label>
                                <input class="form-control" id="validationCustom01" type="text"
                                    placeholder="Blue Ship Pvt Ltd" required="">
                                <!-- <div class="invalid-feedback">Please enter valid name</div>
                                                                                <div class="valid-feedback">
                                                                                    Looks's Good!</div> -->
                            </div>
                            <div class="col-12">
                                <label class="form-label" for="validationCustom01">Product name</label>
                                <input class="form-control" id="validationCustom01" type="text" placeholder="New Ballance"
                                    required="">
                                <!-- <div class="invalid-feedback">Please enter valid name</div>
                                                                                <div class="valid-feedback">
                                                                                    Looks's Good!</div> -->
                            </div>
                            <!-- <div class="col-md-6">
                                                                <label class="form-label" for="validationCustom03">City</label>
                                                                <input class="form-control" id="validationCustom03" type="text" required="">
                                                                <div class="invalid-feedback">Please provide a valid city.</div>
                                                                <div class="valid-feedback">
                                                                    Looks's Good!</div>
                                                            </div> -->
                            <!-- <div class="col-md-6">
                                                                <label class="form-label" for="validationCustom05">Zip</label>
                                                                <input class="form-control" id="validationCustom05" type="text" required="">
                                                                <div class="invalid-feedback">Please provide a valid zip.</div>
                                                                <div class="valid-feedback">
                                                                    Looks's Good!</div>
                                                            </div> -->
                            <div class="col-md-6">
                                <label class="form-label" for="validationCustom03">Quantity</label>
                                <input class="form-control" id="validationCustom03" type="number" required="">
                                <div class="invalid-feedback">Please provide a valid number.</div>
                                <div class="valid-feedback">
                                    Looks's Good!</div>
                            </div>
                            <div class="col-md-6">
                                <label class="form-label" for="validationCustom05">Color</label>
                                <input class="form-control" id="validationCustom05" type="text" required="">
                                <!-- <div class="invalid-feedback">Please provide a valid zip.</div>
                                                                                <div class="valid-feedback">
                                                                                    Looks's Good!</div> -->
                            </div>
                            <div class="col-md-6">
                                <label class="form-label" for="validationCustom04">Product Category</label>
                                <select class="form-select" id="validationCustom04" required="">
                                    <option selected="" disabled="" value="">Choose...</option>
                                    <option>Clothing </option>
                                    <option>Electronic</option>
                                    <option>Sports</option>
                                    <option>Other</option>
                                </select>
                                <div class="invalid-feedback">Please select a valid mode.</div>
                                <div class="valid-feedback">
                                    Looks's Good! </div>
                            </div>
                            <div class="col-md-6">
                                <label class="form-label" for="validationCustom05">Product Price</label>
                                <input class="form-control" id="validationCustom05" type="number" required="">
                                <div class="invalid-feedback">Please provide a valid number.</div>
                                <div class="valid-feedback">
                                    Looks's Good!</div>
                            </div>
                            <div class="col-12">
                                <label class="form-label" for="validationCustom04">Product Status</label>
                                <select class="form-select" id="validationCustom04" required="">
                                    <option selected="" disabled="" value="">Choose...</option>
                                    <option>Pending </option>
                                    <option>In Transit </option>
                                    <option>Delivered</option>
                                    <option>Lost</option>
                                </select>
                                <div class="invalid-feedback">Please select a valid status.</div>
                                <div class="valid-feedback">
                                    Looks's Good! </div>
                            </div>
                            <!-- <div class="col-12">
                                                                            <div class="card-wrapper border rounded-3 checkbox-checked">
                                                                                <h6 class="sub-title">Select your payment method</h6>
                                                                                <div class="radio-form">
                                                                                    <div class="form-check">
                                                                                        <input class="form-check-input" id="validationFormCheck25" type="radio"
                                                                                            name="radio-stacked" required="">
                                                                                        <label class="form-check-label" for="validationFormCheck25">MaterCard</label>
                                                                                    </div>
                                                                                    <div class="form-check">
                                                                                        <input class="form-check-input" id="validationFormCheck23" type="radio"
                                                                                            name="radio-stacked" required="">
                                                                                        <label class="form-check-label" for="validationFormCheck23">VISA</label>
                                                                                    </div>
                                                                                </div>
                                                                            </div>
                                                                        </div> -->
                            <!-- <div class="col-12">
                                                                            <select class="form-select" required="" aria-label="select example">
                                                                                <option value="">Select Your Favorite Pixelstrap theme</option>
                                                                                <option value="1">Cuba</option>
                                                                                <option value="2">Tivo</option>
                                                                                <option value="3">Wingo</option>
                                                                            </select>
                                                                            <div class="invalid-feedback">Invalid select feedback</div>
                                                                        </div> -->
                            <div class="col-12">
                                <label class="form-label" for="formFile1">Legal Documents</label>
                                <input class="form-control" id="formFile1" type="file" aria-label="file example"
                                    required="">
                                <div class="invalid-feedback">Invalid form file selected</div>
                            </div>
                            <div class="col-12">
                                <label class="form-label" for="validationTextarea">Description</label>
                                <textarea class="form-control" id="validationTextarea" placeholder="Enter your comment"
                                    required=""></textarea>
                                <div class="invalid-feedback">Please enter a message in the textarea.</div>
                            </div>
                            <!-- <div class="col-12">
                                                                <div class="form-check">
                                                                    <input class="form-check-input" id="invalidCheck" type="checkbox" value="" required="">
                                                                    <label class="form-check-label" for="invalidCheck">Agree to terms and
                                                                        conditions</label>
                                                                    <div class="invalid-feedback">You must agree before submitting.</div>
                                                                </div>
                                                            </div> -->
                            <div class="col-12">
                                <div class="form-check form-switch">
                                    <input class="form-check-input" id="flexSwitchCheckDefault" type="checkbox"
                                        role="switch" required>
                                    <label class="form-check-label" for="flexSwitchCheckDefault">Agree to terms and
                                        conditions</label>
                                </div>
                            </div>
                            <div class="col-12">
                                <button class="btn btn-primary" type="submit">Save Details</button>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
            <div class="col-xl-4">
                <div class="card h-40 border-0 mb-3">
                    <div class="bg-secondary card-body text-center rounded-4">
                        <h4 class="text-white mb-4">Load Schedule</h4>
                        <label class="form-label text-start w-100" for="validationCustom04" style="text-align: left;">Select
                            Date & Time</label>
                        <button class="form-select btn btn-outline-light mb-4 text-white text-start" data-bs-toggle="modal"
                            data-bs-target="#scheduleModal">
                            Select
                        </button>
                    </div>
                </div>

                <div class="d-flex justify-content-between">
                    <button class="btn btn-outline-primary">Contact Client</button>
                    <button class="btn btn-secondary px-4" data-bs-toggle="modal" data-bs-target="#bidModal">Apply
                        Bid</button>
                </div>

                <!-- Date/Time Selection Modal -->
                <div class="modal fade" id="scheduleModal" tabindex="-1" aria-labelledby="scheduleModalLabel"
                    aria-hidden="true">
                    <div class="modal-dialog modal-dialog-centered modal-lg">
                        <div class="modal-content">
                            <div class="modal-header">
                                <h5 class="modal-title">Choose Date & Time</h5>
                                <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                            </div>
                            <div class="modal-body">
                                <div class="row">
                                    <div class="col-md-6">
                                        <h6 class="text-center mb-3">Sending Date & Time</h6>
                                        <div class="mb-3">
                                            <label class="form-label">Date</label>
                                            <div class="calendar-container">
                                                <div
                                                    class="calendar-header d-flex justify-content-between align-items-center mb-2">
                                                    <button class="btn btn-sm btn-outline-secondary">&lt;</button>
                                                    <h6 class="mb-0">August 2025</h6>
                                                    <button class="btn btn-sm btn-outline-secondary">&gt;</button>
                                                </div>
                                                <table class="table table-bordered">
                                                    <thead>
                                                        <tr>
                                                            <th>Su</th>
                                                            <th>Mo</th>
                                                            <th>Tu</th>
                                                            <th>We</th>
                                                            <th>Th</th>
                                                            <th>Fr</th>
                                                            <th>Sa</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        <tr>
                                                            <td class="text-muted">27</td>
                                                            <td class="text-muted">28</td>
                                                            <td class="text-muted">29</td>
                                                            <td class="text-muted">30</td>
                                                            <td class="text-muted">31</td>
                                                            <td>1</td>
                                                            <td>2</td>
                                                        </tr>
                                                        <tr>
                                                            <td>3</td>
                                                            <td>4</td>
                                                            <td>5</td>
                                                            <td class="bg-primary text-white">6</td>
                                                            <td>7</td>
                                                            <td>8</td>
                                                            <td>9</td>
                                                        </tr>
                                                        <tr>
                                                            <td>10</td>
                                                            <td>11</td>
                                                            <td>12</td>
                                                            <td>13</td>
                                                            <td>14</td>
                                                            <td>15</td>
                                                            <td>16</td>
                                                        </tr>
                                                        <tr>
                                                            <td>17</td>
                                                            <td>18</td>
                                                            <td>19</td>
                                                            <td>20</td>
                                                            <td>21</td>
                                                            <td>22</td>
                                                            <td>23</td>
                                                        </tr>
                                                        <tr>
                                                            <td>24</td>
                                                            <td>25</td>
                                                            <td>26</td>
                                                            <td>27</td>
                                                            <td>28</td>
                                                            <td>29</td>
                                                            <td>30</td>
                                                        </tr>
                                                        <tr>
                                                            <td class="text-muted">31</td>
                                                            <td class="text-muted">1</td>
                                                            <td class="text-muted">2</td>
                                                            <td class="text-muted">3</td>
                                                            <td class="text-muted">4</td>
                                                            <td class="text-muted">5</td>
                                                            <td class="text-muted">6</td>
                                                        </tr>
                                                    </tbody>
                                                </table>
                                            </div>
                                        </div>
                                        <div class="row">
                                            <div class="col-6">
                                                <label class="form-label">Hour</label>
                                                <select class="form-select">
                                                    <option>08</option>
                                                    <option>09</option>
                                                    <option>10</option>
                                                    <option>11</option>
                                                    <option>12</option>
                                                    <option>01</option>
                                                    <option>02</option>
                                                </select>
                                            </div>
                                            <div class="col-6">
                                                <label class="form-label">Minute</label>
                                                <select class="form-select">
                                                    <option>00</option>
                                                    <option>15</option>
                                                    <option>30</option>
                                                    <option>45</option>
                                                </select>
                                            </div>
                                            <div class="col-12 mt-2">
                                                <div class="btn-group w-100">
                                                    <button class="btn btn-outline-primary active">AM</button>
                                                    <button class="btn btn-outline-primary">PM</button>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                    <div class="col-md-6">
                                        <h6 class="text-center mb-3">Receiving Date & Time</h6>
                                        <div class="mb-3">
                                            <label class="form-label">Date</label>
                                            <div class="calendar-container">
                                                <div
                                                    class="calendar-header d-flex justify-content-between align-items-center mb-2">
                                                    <button class="btn btn-sm btn-outline-secondary">&lt;</button>
                                                    <h6 class="mb-0">August 2025</h6>
                                                    <button class="btn btn-sm btn-outline-secondary">&gt;</button>
                                                </div>
                                                <table class="table table-bordered">
                                                    <thead>
                                                        <tr>
                                                            <th>Su</th>
                                                            <th>Mo</th>
                                                            <th>Tu</th>
                                                            <th>We</th>
                                                            <th>Th</th>
                                                            <th>Fr</th>
                                                            <th>Sa</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        <tr>
                                                            <td class="text-muted">27</td>
                                                            <td class="text-muted">28</td>
                                                            <td class="text-muted">29</td>
                                                            <td class="text-muted">30</td>
                                                            <td class="text-muted">31</td>
                                                            <td>1</td>
                                                            <td>2</td>
                                                        </tr>
                                                        <tr>
                                                            <td>3</td>
                                                            <td>4</td>
                                                            <td>5</td>
                                                            <td>6</td>
                                                            <td>7</td>
                                                            <td>8</td>
                                                            <td>9</td>
                                                        </tr>
                                                        <tr>
                                                            <td>10</td>
                                                            <td>11</td>
                                                            <td>12</td>
                                                            <td>13</td>
                                                            <td>14</td>
                                                            <td>15</td>
                                                            <td>16</td>
                                                        </tr>
                                                        <tr>
                                                            <td>17</td>
                                                            <td>18</td>
                                                            <td>19</td>
                                                            <td>20</td>
                                                            <td>21</td>
                                                            <td>22</td>
                                                            <td>23</td>
                                                        </tr>
                                                        <tr>
                                                            <td>24</td>
                                                            <td>25</td>
                                                            <td>26</td>
                                                            <td>27</td>
                                                            <td>28</td>
                                                            <td>29</td>
                                                            <td>30</td>
                                                        </tr>
                                                        <tr>
                                                            <td class="text-muted">31</td>
                                                            <td class="text-muted">1</td>
                                                            <td class="text-muted">2</td>
                                                            <td class="text-muted">3</td>
                                                            <td class="text-muted">4</td>
                                                            <td class="text-muted">5</td>
                                                            <td class="text-muted">6</td>
                                                        </tr>
                                                    </tbody>
                                                </table>
                                            </div>
                                        </div>
                                        <div class="row">
                                            <div class="col-6">
                                                <label class="form-label">Hour</label>
                                                <select class="form-select">
                                                    <option>08</option>
                                                    <option>09</option>
                                                    <option>10</option>
                                                    <option>11</option>
                                                    <option>12</option>
                                                    <option>01</option>
                                                    <option>02</option>
                                                </select>
                                            </div>
                                            <div class="col-6">
                                                <label class="form-label">Minute</label>
                                                <select class="form-select">
                                                    <option>00</option>
                                                    <option>15</option>
                                                    <option>30</option>
                                                    <option>45</option>
                                                </select>
                                            </div>
                                            <div class="col-12 mt-2">
                                                <div class="btn-group w-100">
                                                    <button class="btn btn-outline-primary">AM</button>
                                                    <button class="btn btn-outline-primary active">PM</button>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <div class="modal-footer">
                                <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">Clear</button>
                                <button type="button" class="btn btn-primary">Save</button>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Apply Bid Modal -->
                <div class="modal fade" id="bidModal" tabindex="-1" aria-labelledby="bidModalLabel" aria-hidden="true">
                    <div class="modal-dialog modal-dialog-centered">
                        <div class="modal-content">
                            <div class="modal-header">
                                <h5 class="modal-title">Apply Bid</h5>
                                <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                            </div>
                            <div class="modal-body">
                                <form>
                                    <div class="mb-3">
                                        <label for="bidAmount" class="form-label">Bid Amount ($)</label>
                                        <input type="number" class="form-control" id="bidAmount"
                                            placeholder="Enter your bid amount">
                                    </div>
                                    <div class="mb-3">
                                        <label for="bidNotes" class="form-label">Notes (Optional)</label>
                                        <textarea class="form-control" id="bidNotes" rows="3"
                                            placeholder="Any additional information..."></textarea>
                                    </div>
                                    <div class="form-check mb-3">
                                        <input class="form-check-input" type="checkbox" id="termsCheck">
                                        <label class="form-check-label" for="termsCheck">
                                            I agree to the terms and conditions
                                        </label>
                                    </div>
                                </form>
                            </div>
                            <div class="modal-footer">
                                <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">Cancel</button>
                                <button type="button" class="btn btn-primary">Submit Bid</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>



        </div>
    </div>
    <!-- Container-fluid Ends-->
@endsection