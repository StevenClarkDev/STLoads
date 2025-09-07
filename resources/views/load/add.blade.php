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
                            {{-- <div class="col-md-6">
                                <label class="form-label" for="formFile1">Documents</label>
                                <input class="form-control" id="formFile1" type="file" name="documents"
                                    accept=".pdf,.jpg,.jpeg,.png,.webp,.doc,.docx" />
                                <div class="invalid-feedback">Invalid form file selected</div>
                            </div> --}}
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
                            <div class="col-mb-6 mb-3">
                                <label class="form-label" for="validationTextarea">Special Instructions</label>
                                <textarea class="form-control" id="validationTextarea" name="special_instructions"
                                    placeholder="Enter your Special Instructions"></textarea>
                                <div class="invalid-feedback">Please enter a message in the textarea.</div>
                            </div>

                            <div class="col-xl-12">
                                <div class="card">
                                    <div class="card-header d-flex">
                                        <h4 class="mb-0">Documents</h4>
                                        <button type="button" class="btn btn-primary btn-sm" id="doc-row">
                                            <i class="bi bi-plus-lg"></i> Add
                                        </button>
                                    </div>
                                    <div class="card-body">
                                        <div class="table-responsive">
                                            <table class="table table-bordered" id="document-table">
                                                <thead>
                                                    <tr>
                                                        <th>#</th>
                                                        <th>Ducument Name</th>
                                                        <th>Ducument Type</th>
                                                        <th>Document</th>
                                                        <th>Action</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                </tbody>
                                            </table>
                                        </div>
                                    </div>
                                </div>
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



<script src="{{ url('assets/js/jquery.min.js') }}"></script>

<script>
    $(function() {
        // ----- Server-rendered snippets (NO Blade inside JS strings below) -----
        const locationOptions = `{!! collect($locations)->map(function ($l) {
                $label = trim(($l->name ?? '') . ' ' . ($l->city->name ?? '') . ' ' . ($l->country->name ?? ''));
                return '<option value="' . $l->id . '">' . e($label) . '</option>';
            })->implode('') !!}`;

        const canEditLegs = {{ $roleId == 5 ? 'true' : 'false' }};

        function renumberRows() {
            $('#load_legs-table tbody tr').each(function(i) {
                $(this).find('input.leg-id').val(i + 1);
            });
        }

        // ----- Row template (pure JS; inject prebuilt options) -----
        function rowTemplate() {
            return `
        <tr>
          <td><input type="text" name="leg_id[]" class="form-control leg-id" value="" readonly /></td>

          <td>
            <select name="pickup_location[]" class="form-select" required>
              <option value="">Select...</option>
              ${locationOptions}
            </select>
          </td>

          <td>
            <select name="delivery_location[]" class="form-select" required>
              <option value="">Select...</option>
              ${locationOptions}
            </select>
          </td>

          <td>
            <div class="input-group flatpicker-calender">
              <input class="form-control datepicker" name="pickup_date[]" type="text" required>
            </div>
          </td>

          <td>
            <div class="input-group flatpicker-calender">
              <input class="form-control datepicker" name="delivery_date[]" type="date" required>
            </div>
          </td>

          <td>
            <select name="bid_status[]" class="form-select" required>
              <option value="Fixed">Fixed</option>
              <option value="Open">Open</option>
            </select>
          </td>

          <td><input type="number" min="0" name="price[]" class="form-control" required /></td>

          ${canEditLegs ? `
            <td>
              <button type="button" class="btn-remove-icon remove-row-load_legs" title="Remove">
                <i class="bi bi-trash"></i>
              </button>
            </td>
          ` : ``}
        </tr>
      `;
        }

        function addRow() {
            const $row = $(rowTemplate());
            $('#load_legs-table tbody').append($row);
            renumberRows();
        }

        // ----- Events -----
        $('#add-load_legs-row').on('click', addRow);

        $('#load_legs-table').on('click', '.remove-row-load_legs', function() {
            $(this).closest('tr').remove();
            renumberRows();
        });

        // ----- Init -----
        addRow();
    });
</script>
<script>
    $(document).ready(function() {
        function addMemberRow() {
            const rowCount = $('#document-table tbody tr').length + 1;
            const newRow = `
            <tr>
                <td>${rowCount}</td>
                <td><input type="text" name="doc_name[]" class="form-control" required /></td>
                <td>
                    <select name="doc_type[]" required class="form-control">
                        <option value="standard">Standard</option>
                        <option value="blockchain">Blockchain</option>
                    </select>
                </td>
                <td>
                    <input type="file" name="documents[]" class="form-control"
                           accept=".pdf,.jpg,.jpeg,.png,.docx,application/pdf,application/vnd.openxmlformats-officedocument.wordprocessingml.document,image/jpeg,image/png"
                           required />
                </td>
                <td><button type="button" class="btn btn-danger remove-row">Remove</button></td>
            </tr>`;
            $('#document-table tbody').append(newRow);
            updateSerialNumbers('#document-table');
            toggleRemoveButtons();
        }

        function updateSerialNumbers(tableId) {
            $(tableId + ' tbody tr').each(function(index) {
                $(this).find('td:first').text(index + 1);
            });
        }

        function toggleRemoveButtons() {
            const rows = $('#document-table tbody tr');
            // prevent deleting the last remaining row
            rows.find('.remove-row').prop('disabled', rows.length === 1);
        }

        $('#doc-row').on('click', addMemberRow);

        $('body').on('click', '.remove-row', function() {
            $(this).closest('tr').remove();
            updateSerialNumbers('#document-table');
            toggleRemoveButtons();
            if ($('#document-table tbody tr').length === 0) addMemberRow();
        });

        // start with one row
        addMemberRow();
    });
</script>
