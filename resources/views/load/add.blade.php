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
            <div class="col-xl-12">
                <div class="card height-equal">
                    <div class="card-header">
                        <h4>Load Details</h4>
                    </div>
                    <div class="card-body">
                        <form class="row g-3 needs-validation custom-input" novalidate="">
                            <div class="col-md-6">
                                <label class="form-label" for="title">Title</label>
                                <input class="form-control" id="title" name="title" type="text" required>
                            </div>
                            <div class="col-md-6">
                                <label class="form-label" for="load_type">Load Type</label>
                                <select class="form-select" id="load_type" name="load_type" required>
                                    <option selected value="">Choose...</option>
                                    @foreach ($load_types as $load_type)
                                        <option value="{{ $load_type->id }}">{{ $load_type->name }}</option>
                                    @endforeach
                                </select>
                            </div>
                            <div class="col-md-6">
                                <label class="form-label" for="equipment_required">Equipment Required</label>
                                <select class="form-select" id="equipment_required" name="equipment_required" required>
                                    <option selected value="">Choose...</option>
                                    @foreach ($equipments as $equipment)
                                        <option value="{{ $equipment->id }}">{{ $equipment->name }}</option>
                                    @endforeach
                                </select>
                            </div>
                            <div class="col-md-6">
                                <label class="form-label" for="weight_unit">WT Unit</label>
                                <select class="form-select" id="weight_unit" name="weight_unit" required>
                                    <option selected value="">Choose...</option>
                                    <option value="LBS">LBS</option>
                                    <option value="KG">KG</option>
                                    <option value="MTON">MTON</option>
                                </select>
                            </div>
                            <div class="col-md-6">
                                <label class="form-label" for="weight">Weight</label>
                                <input class="form-control" id="weight" type="number" name="weight" required>
                            </div>
                            <div class="col-md-6">
                                <label class="form-label" for="commodity_type">Commodity Type</label>
                                <select class="form-select" id="commodity_type" name="commodity_type" required>
                                    <option selected value="">Choose...</option>
                                    @foreach ($commodity_types as $commodity_type)
                                        <option value="{{ $commodity_type->id }}">{{ $commodity_type->name }}</option>
                                    @endforeach
                                </select>
                            </div>
                            <div class="col-md-6">
                                <label class="form-label" for="formFile1">Documents</label>
                                <input class="form-control" id="formFile1" type="file" name="documents"
                                    aria-label="file example">
                                <div class="invalid-feedback">Invalid form file selected</div>
                            </div>
                            <div class="col-md-6">
                                <div class="d-flex flex-row mt-4">
                                    <div class="form-check my-2 me-4">
                                        <input class="form-check-input" type="checkbox" id="is_hazardous"
                                            name="is_hazardous">
                                        <label for="is_hazardous">
                                            Is Hazardous Material?
                                        </label>
                                    </div>
                                    <div class="form-check my-2">
                                        <input class="form-check-input" type="checkbox" id="is_temperature_controlled"
                                            name="is_temperature_controlled">
                                        <label for="is_temperature_controlled">
                                            Is Temperature Controlled?
                                        </label>
                                    </div>
                                </div>
                            </div>
                            <div class="col-mb-12 mb-3">
                                <label class="form-label" for="validationTextarea">Special Instructions</label>
                                <textarea class="form-control" id="validationTextarea"
                                    placeholder="Enter your Special Instructions"></textarea>
                                <div class="invalid-feedback">Please enter a message in the textarea.</div>
                            </div>
                            <div class="col-xl-12">
                                <div class="card">
                                    <div class="card-header pb-0">
                                        <h4 class="mb-3">Load Legs</h4>
                                    </div>
                                    <div class="card-body">
                                        <div class="basic_table" id="basicScenario"></div>
                                    </div>
                                </div>
                                <!-- <div class="card h-40 border-0 mb-3">
                                        <div class="bg-secondary card-body text-center rounded-4">
                                            <h4 class="text-white mb-4">Load Legs</h4>
                                            <h6 class="text-white mb-4">Insert Table here</h6>
                                            <p>tables will contain columns like Origin Address (dropdown), Destination Address
                                                (dropdown), Pickup Date, Delivery Date, is bidable or fixed price? (checkbox),
                                                Price</p>
                                            {{-- <label class="form-label text-start w-100" for="validationCustom04"
                                                style="text-align: left;">Select
                                                Date & Time</label> --}}
                                            {{-- <button class="form-select btn btn-outline-light mb-4 text-white text-start"
                                                data-bs-toggle="modal" data-bs-target="#scheduleModal">
                                                Select
                                            </button> --}}
                                        </div>
                                    </div> -->

                                {{-- <div class="d-flex justify-content-between">
                                    <button class="btn btn-outline-primary">Contact Client</button>
                                    <button class="btn btn-secondary px-4" data-bs-toggle="modal"
                                        data-bs-target="#bidModal">Apply
                                        Bid</button>
                                </div> --}}

                                <!-- Date/Time Selection Modal -->
                                {{-- <div class="modal fade" id="scheduleModal" tabindex="-1"
                                    aria-labelledby="scheduleModalLabel" aria-hidden="true">
                                    <div class="modal-dialog modal-dialog-centered modal-lg">
                                        <div class="modal-content">
                                            <div class="modal-header">
                                                <h5 class="modal-title">Choose Date & Time</h5>
                                                <button type="button" class="btn-close" data-bs-dismiss="modal"
                                                    aria-label="Close"></button>
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
                                                                    <button
                                                                        class="btn btn-sm btn-outline-secondary">&lt;</button>
                                                                    <h6 class="mb-0">August 2025</h6>
                                                                    <button
                                                                        class="btn btn-sm btn-outline-secondary">&gt;</button>
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
                                                                    <button
                                                                        class="btn btn-outline-primary active">AM</button>
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
                                                                    <button
                                                                        class="btn btn-sm btn-outline-secondary">&lt;</button>
                                                                    <h6 class="mb-0">August 2025</h6>
                                                                    <button
                                                                        class="btn btn-sm btn-outline-secondary">&gt;</button>
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
                                                                    <button
                                                                        class="btn btn-outline-primary active">PM</button>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                            <div class="modal-footer">
                                                <button type="button" class="btn btn-secondary"
                                                    data-bs-dismiss="modal">Clear</button>
                                                <button type="button" class="btn btn-primary">Save</button>
                                            </div>
                                        </div>
                                    </div>
                                </div> --}}

                                <!-- Apply Bid Modal -->
                                {{-- <div class="modal fade" id="bidModal" tabindex="-1" aria-labelledby="bidModalLabel"
                                    aria-hidden="true">
                                    <div class="modal-dialog modal-dialog-centered">
                                        <div class="modal-content">
                                            <div class="modal-header">
                                                <h5 class="modal-title">Apply Bid</h5>
                                                <button type="button" class="btn-close" data-bs-dismiss="modal"
                                                    aria-label="Close"></button>
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
                                                <button type="button" class="btn btn-secondary"
                                                    data-bs-dismiss="modal">Cancel</button>
                                                <button type="button" class="btn btn-primary">Submit Bid</button>
                                            </div>
                                        </div>
                                    </div>
                                </div> --}}
                            </div>

                            <div class="col-12">
                                <button class="btn btn-primary" type="submit">Save Details</button>
                            </div>
                        </form>
                    </div>
                </div>
            </div>

        </div>
    </div>
    <!-- Container-fluid Ends-->
@endsection