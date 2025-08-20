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
                        <form class="row g-3 custom-input" action="{{ route('loads.store') }}" method="POST"
                            enctype="multipart/form-data">
                            @csrf
                            <div class="col-md-6">
                                <label class="form-label" for="title">Title</label>
                                <input class="form-control" id="title" name="title" type="text" required>
                                <input id="user_id" name="user_id" type="hidden" value="{{ $user_id }}">
                                <input id="role_id" name="role_id" type="hidden" value="{{ $roleId }}">
                            </div>
                            <div class="col-md-6">
                                <label class="form-label" for="load_type">Load Type</label>
                                <select class="form-select" id="load_type" name="load_type_id" required>
                                    <option selected value="">Choose...</option>
                                    @foreach ($load_types as $load_type)
                                        <option value="{{ $load_type->id }}">{{ $load_type->name }}</option>
                                    @endforeach
                                </select>
                            </div>
                            <div class="col-md-6">
                                <label class="form-label" for="equipment_required">Equipment Required</label>
                                <select class="form-select" id="equipment_required" name="equipment_id" required>
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
                                <select class="form-select" id="commodity_type" name="commodity_type_id" required>
                                    <option selected value="">Choose...</option>
                                    @foreach ($commodity_types as $commodity_type)
                                        <option value="{{ $commodity_type->id }}">{{ $commodity_type->name }}</option>
                                    @endforeach
                                </select>
                            </div>
                            <div class="col-md-6">
                                <label class="form-label" for="formFile1">Documents</label>
                                <input class="form-control" id="formFile1" type="file" name="documents"
                                    accept=".pdf,.jpg,.jpeg,.png,.webp,.doc,.docx" />
                                <div class="invalid-feedback">Invalid form file selected</div>
                            </div>
                            <div class="col-md-6">
                                <div class="d-flex flex-row mt-4">
                                    <div class="form-check my-2 me-4">
                                        <input class="form-check-input" type="checkbox" id="is_hazardous"
                                            name="is_hazardous">
                                        <label for="is_hazardous">Is Hazardous Material?</label>
                                    </div>
                                    <div class="form-check my-2">
                                        <input class="form-check-input" type="checkbox" id="is_temperature_controlled"
                                            name="is_temperature_controlled">
                                        <label for="is_temperature_controlled">Is Temperature Controlled?</label>
                                    </div>
                                </div>
                            </div>
                            <div class="col-mb-12 mb-3">
                                <label class="form-label" for="validationTextarea">Special Instructions</label>
                                <textarea class="form-control" id="validationTextarea" name="special_instructions"
                                    placeholder="Enter your Special Instructions"></textarea>
                                <div class="invalid-feedback">Please enter a message in the textarea.</div>
                            </div>

                            <!-- Load Legs Table -->
                            <div class="col-xl-12">
                                <div class="card">
                                    <div class="card-header d-flex">
                                        <h4 class="mb-0">Load Legs</h4>
                                        @if ($roleId == 5)
                                            <button type="button" class="btn btn-primary btn-sm" id="add-load_legs-row">
                                                <i class="bi bi-plus-lg"></i> Add
                                            </button>
                                        @endif
                                    </div>
                                    <div class="card-body">
                                        <div class="table-responsive">
                                            <table class="table table-bordered" id="load_legs-table">
                                                <thead class="table-light">
                                                    <tr>
                                                        <th>S.No</th>
                                                        <th>Pickup Location</th>
                                                        <th>Delivery Location</th>
                                                        <th>Pickup Date</th>
                                                        <th>Delivery Date</th>
                                                        <th>Bid Status</th>
                                                        <th>Price</th>
                                                        @if ($roleId == 5)
                                                            <th>Action</th>
                                                        @endif
                                                    </tr>
                                                </thead>
                                                <tbody></tbody>
                                            </table>
                                        </div>
                                    </div>
                                </div>
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

<style>
    .card-header.d-flex {
        align-items: center;
        justify-content: space-between;
    }

    .table-responsive {
        max-height: 250px;
        overflow-y: auto;
        overflow-x: auto;
    }

    #load_legs-table th,
    #load_legs-table td {
        white-space: nowrap;
        text-align: center;
        vertical-align: middle;
    }

    #load_legs-table .form-control,
    #load_legs-table .form-select {
        min-width: 150px;
        padding: 4px 6px;
        height: 36px;
    }

    #load_legs-table td:first-child,
    #load_legs-table th:first-child,
    #load_legs-table td:nth-last-child(2),
    #load_legs-table th:nth-last-child(2) {
        width: 80px;
    }

    .btn-remove-icon {
        background: none;
        border: none;
        padding: 0;
        color: #dc3545;
        font-size: 1.2rem;
        cursor: pointer;
    }

    .btn-remove-icon:hover {
        color: #a71d2a;
    }
</style>

<script src="{{ url('assets/js/jquery.min.js') }}"></script>
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/flatpickr/dist/flatpickr.min.css">
<script src="https://cdn.jsdelivr.net/npm/flatpickr"></script>

<script>
  $(function () {
    // Init flatpickr (optionally scoped to a container)
    function initDatePickers(scope) {
      const ctx = scope || document;
      $(ctx).find(".datetimepicker").each(function () {
        flatpickr(this, {
          enableTime: true,
          dateFormat: "Y-m-d H:i",
        });
      });
    }

    // Table row HTML (Blade will render the @foreach options server-side)
    function rowTemplate() {
      return `
        <tr>
          <td><input type="text" name="leg_id[]" class="form-control leg-id" value="" readonly /></td>

          <td>
            <select name="pickup_location[]" class="form-select" required>
              <option value="">Select...</option>
              @foreach ($locations as $location)
                <option value="{{ $location->id }}">
                  {{ $location->name }} {{ $location->city->name }} {{ $location->country->name }}
                </option>
              @endforeach
            </select>
          </td>

          <td>
            <select name="delivery_location[]" class="form-select" required>
              <option value="">Select...</option>
              @foreach ($locations as $location)
                <option value="{{ $location->id }}">
                  {{ $location->name }} {{ $location->city->name }} {{ $location->country->name }}
                </option>
              @endforeach
            </select>
          </td>

          <td>
            <div class="input-group flatpicker-calender">
              <input class="form-control datetimepicker" name="pickup_date[]" type="text" required>
            </div>
          </td>
          <td>
            <div class="input-group flatpicker-calender">
              <input class="form-control datetimepicker" name="delivery_date[]" type="text" required>
            </div>
          </td>

          <td>
            <select name="bid_status[]" class="form-select" required>
              <option value="Fixed">Fixed</option>
              <option value="Open">Open</option>
            </select>
          </td>

          <td><input type="number" min="0" name="price[]" class="form-control" required /></td>

          @if ($roleId == 5)
          <td>
            <button type="button" class="btn-remove-icon remove-row-load_legs" title="Remove">
              <i class="bi bi-trash"></i>
            </button>
          </td>
          @endif
        </tr>
      `;
    }

    // Re-number S.No column
    function renumberRows() {
      $('#load_legs-table tbody tr').each(function (i) {
        $(this).find('input.leg-id').val(i + 1);
      });
    }

    // Add a new row
    function addRow() {
      const $row = $(rowTemplate());
      $('#load_legs-table tbody').append($row);
      initDatePickers($row); // init pickers only in the new row
      renumberRows();
    }

    // Handlers
    $('#add-load_legs-row').on('click', function () {
      addRow();
    });

    $('#load_legs-table').on('click', '.remove-row-load_legs', function () {
      $(this).closest('tr').remove();
      renumberRows();
    });

    // Initial state
    addRow();
  });
</script>

