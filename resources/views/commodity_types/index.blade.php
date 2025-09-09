@extends('admin-layout.app')

@section('content')
    <div class="row">
        <div class="col-xl-12 box-col-6 px-3 py-2">
            <div class="card">
                <div class="card-body p-0">
                    <div class="card mx-3">
                        <div class="card-header pb-0 card-no-border d-flex justify-content-between align-items-center">
                            <div>
                                <h4>Commodity Types List</h4>
                                <span>See Commodity Types below.</span>
                            </div>
                            <!-- Add Button opens Add Modal -->
                            <button type="button" class="btn btn-primary btn-sm" data-bs-toggle="modal"
                                data-bs-target="#addCommodityTypeModal">
                                <i class="fa fa-plus"></i> Add Commodity Type
                            </button>
                        </div>

                        <div class="card-body">
                            <div class="table-responsive">
                                <div style="max-height:500px;">
                                    <table class="table table-striped w-100" id="user-approval-table">
                                        <thead style="position: sticky; top: 0; background: #fff; z-index: 2;">
                                            <tr>
                                                <th>S No.</th>
                                                <th>Name</th>
                                                <th>Action</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            @foreach ($commodity_types as $i => $commodity_type)
                                                <tr>
                                                    <td>{{ ++$i }}</td>
                                                    <td>{{ $commodity_type->name }}</td>
                                                    <td class="d-flex gap-1">
                                                        <!-- Edit -->
                                                        <button type="button" class="btn btn-primary btn-sm editBtn"
                                                            data-id="{{ $commodity_type->id }}"
                                                            data-name="{{ $commodity_type->name }}"
                                                            data-action="{{ route('commodity_types.update', $commodity_type->id) }}">
                                                            <i class="fa fa-pencil"></i> Edit
                                                        </button>
                                                        <!-- Delete -->
                                                        <button type="button" class="btn btn-danger btn-sm deleteBtn"
                                                            data-id="{{ $commodity_type->id }}"
                                                            data-name="{{ $commodity_type->name }}"
                                                            data-action="{{ route('commodity_types.destroy', $commodity_type->id) }}">
                                                            <i class="fa fa-trash"></i> Delete
                                                        </button>
                                                    </td>
                                                </tr>
                                            @endforeach
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        </div>

                    </div> <!-- /.card mx-3 -->
                </div>
            </div>
        </div>
    </div>

    <!-- ADD Commodity Type Modal -->
    <div class="modal fade" id="addCommodityTypeModal" tabindex="-1" aria-hidden="true">
        <div class="modal-dialog modal-dialog-centered" style="max-width: 500px;">
            <div class="modal-content p-4">
                <div class="modal-header border-0">
                    <h5 class="modal-title">Add Commodity Type</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <form method="POST" action="{{ route('commodity_types.store') }}">
                        @csrf
                        <div class="mb-3">
                            <label class="form-label">Name</label>
                            <input type="text" class="form-control" name="name" placeholder="Enter Commodity Type" required>
                        </div>
                        <div class="text-end">
                            <button type="submit" class="btn btn-primary btn-sm px-4">Save</button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    </div>

    <!-- EDIT Commodity Type Modal -->
    <div class="modal fade" id="editCommodityTypeModal" tabindex="-1" aria-hidden="true">
        <div class="modal-dialog modal-dialog-centered" style="max-width: 500px;">
            <div class="modal-content p-4">
                <div class="modal-header border-0">
                    <h5 class="modal-title">Edit Commodity Type</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <form id="editCommodityTypeForm" method="POST" action="#">
                        @csrf
                        @method('PUT')
                        <div class="mb-3">
                            <label class="form-label">Name</label>
                            <input type="text" class="form-control" id="edit-name" name="name" required>
                        </div>
                        <div class="text-end">
                            <button type="submit" class="btn btn-primary btn-sm px-4">Update</button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    </div>

    <!-- DELETE Confirmation Modal -->
    <div class="modal fade" id="deleteCommodityTypeModal" tabindex="-1" aria-hidden="true">
        <div class="modal-dialog modal-dialog-centered" style="max-width: 450px;">
            <div class="modal-content p-4">
                <div class="modal-header border-0">
                    <h5 class="modal-title text-danger">Confirm Delete</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
                </div>
                <div class="modal-body">
                    <p>Are you sure you want to delete <strong id="delete-item-name"></strong>?</p>
                    <form id="deleteCommodityTypeForm" method="POST" action="#">
                        @csrf
                        @method('DELETE')
                        <div class="text-end">
                            <button type="button" class="btn btn-secondary btn-sm" data-bs-dismiss="modal">Cancel</button>
                            <button type="submit" class="btn btn-danger btn-sm px-4">Delete</button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    </div>

    {{-- Inline Script --}}
    <script>
        document.addEventListener('click', function (e) {
            // EDIT
            if (e.target.closest('.editBtn')) {
                const btn = e.target.closest('.editBtn');
                const name = btn.getAttribute('data-name');
                const action = btn.getAttribute('data-action');

                document.getElementById('edit-name').value = name || '';
                document.getElementById('editCommodityTypeForm').setAttribute('action', action);

                const modalEl = document.getElementById('editCommodityTypeModal');
                bootstrap.Modal.getOrCreateInstance(modalEl).show();
            }

            // DELETE
            if (e.target.closest('.deleteBtn')) {
                const btn = e.target.closest('.deleteBtn');
                const name = btn.getAttribute('data-name');
                const action = btn.getAttribute('data-action');

                document.getElementById('delete-item-name').textContent = name || '';
                document.getElementById('deleteCommodityTypeForm').setAttribute('action', action);

                const modalEl = document.getElementById('deleteCommodityTypeModal');
                bootstrap.Modal.getOrCreateInstance(modalEl).show();
            }
        });
    </script>
@endsection