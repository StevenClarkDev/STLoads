@extends('admin-layout.app')

@section('content')
    @php
        // Pull countries here so we don't need to modify the controller.
        $countries = \App\Models\Country::orderBy('name')->get(['id', 'name']);
    @endphp

    <div class="row">
        <div class="col-xl-12 box-col-6 px-3 py-2">
            <div class="card">
                <div class="card-body p-0">
                    <div class="card mx-3">

                        <!-- Header -->
                        <div class="card-header pb-0 card-no-border d-flex justify-content-between align-items-center">
                            <div>
                                <h4>Location List</h4>
                                <span>See Location below.</span>
                            </div>
                            <button type="button" class="btn btn-primary btn-sm" data-bs-toggle="modal"
                                data-bs-target="#addLocationModal">
                                <i class="fa fa-plus"></i> Add Location
                            </button>
                        </div>

                        <!-- Table -->
                        <div class="card-body">
                            <div class="table-responsive" style="max-height:500px;">
                                <table class="table table-striped w-100" id="user-approval-table">
                                    <thead style="position: sticky; top: 0; background: #fff; z-index: 2;">
                                        <tr>
                                            <th>S No.</th>
                                            <th>Address</th>
                                            <th>Country</th>
                                            <th>City</th>
                                            <th>Action</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        @foreach ($locations as $i => $location)
                                            <tr>
                                                <td>{{ ++$i }}</td>
                                                <td>{{ $location->name }}</td>
                                                <td>{{ $location->country?->name }}</td>
                                                <td>{{ $location->city?->name }}</td>
                                                <td class="d-flex gap-1">
                                                    <!-- Edit Button -->
                                                    <button type="button" class="btn btn-primary btn-sm editBtn"
                                                        data-id="{{ $location->id }}" data-name="{{ $location->name }}"
                                                        data-country-id="{{ $location->country_id }}"
                                                        data-city-id="{{ $location->city_id }}"
                                                        data-action="{{ route('locations.update', $location->id) }}">
                                                        <i class="fa fa-pencil"></i> Edit
                                                    </button>

                                                    <!-- Delete Button -->
                                                    <button type="button" class="btn btn-danger btn-sm deleteBtn"
                                                        data-id="{{ $location->id }}" data-name="{{ $location->name }}"
                                                        data-action="{{ route('locations.destroy', $location->id) }}">
                                                        <i class="fa fa-trash"></i> Delete
                                                    </button>
                                                </td>
                                            </tr>
                                        @endforeach
                                    </tbody>
                                </table>
                            </div>
                        </div>
                        <!-- /Table -->

                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- =========================
             ADD Location Modal
        ========================== -->
    <div class="modal fade" id="addLocationModal" tabindex="-1" aria-hidden="true">
        <div class="modal-dialog modal-dialog-centered" style="max-width: 650px;">
            <div class="modal-content p-4">
                <div class="modal-header border-0">
                    <h5 class="modal-title">Add Location</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal"></button>
                </div>

                <div class="modal-body">
                    <form method="POST" action="{{ route('locations.store') }}" id="addLocationForm">
                        @csrf
                        <div class="row g-4">
                            <div class="col-md-12">
                                <div class="form-floating form-floating-outline">
                                    <input type="text" class="form-control" id="add-name" name="name"
                                        placeholder="Enter Name" required>
                                    <label for="add-name">Address</label>
                                </div>
                            </div>

                            <div class="col-md-6">
                                <div class="form-floating form-floating-outline">
                                    <select name="country_id" class="form-select country-select" id="add-country" required>
                                        <option value="">-- Select Country --</option>
                                        @foreach($countries as $country)
                                            <option value="{{ $country->id }}">{{ $country->name }}</option>
                                        @endforeach
                                    </select>
                                    <label for="add-country">Country</label>
                                </div>
                            </div>

                            <div class="col-md-6">
                                <div class="form-floating form-floating-outline">
                                    <select name="city_id" class="form-select city-select" id="add-city" disabled required>
                                        <option value="">-- Select City --</option>
                                    </select>
                                    <label for="add-city">City</label>
                                </div>
                            </div>
                        </div>

                        <div class="d-flex flex-row-reverse gap-1 mt-3">
                            <button type="submit" class="btn btn-outline-primary">Submit</button>
                            <button type="button" class="btn btn-outline-secondary" data-bs-dismiss="modal">Cancel</button>
                        </div>
                    </form>
                </div>

            </div>
        </div>
    </div>

    <!-- =========================
             EDIT Location Modal
        ========================== -->
    <div class="modal fade" id="editLocationModal" tabindex="-1" aria-hidden="true">
        <div class="modal-dialog modal-dialog-centered" style="max-width: 650px;">
            <div class="modal-content p-4">
                <div class="modal-header border-0">
                    <h5 class="modal-title">Edit Location</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal"></button>
                </div>

                <div class="modal-body">
                    <form id="editLocationForm" method="POST" action="#">
                        @csrf
                        @method('PUT')

                        <div class="row g-4">
                            <div class="col-md-12">
                                <div class="form-floating form-floating-outline">
                                    <input type="text" class="form-control" id="edit-name" name="name"
                                        placeholder="Enter Name" required>
                                    <label for="edit-name">Address</label>
                                </div>
                            </div>

                            <div class="col-md-6">
                                <div class="form-floating form-floating-outline">
                                    <select name="country_id" class="form-select country-select" id="edit-country" required>
                                        <option value="">-- Select Country --</option>
                                        @foreach($countries as $country)
                                            <option value="{{ $country->id }}">{{ $country->name }}</option>
                                        @endforeach
                                    </select>
                                    <label for="edit-country">Country</label>
                                </div>
                            </div>

                            <div class="col-md-6">
                                <div class="form-floating form-floating-outline">
                                    <select name="city_id" class="form-select city-select" id="edit-city" disabled required>
                                        <option value="">-- Select City --</option>
                                    </select>
                                    <label for="edit-city">City</label>
                                </div>
                            </div>
                        </div>

                        <div class="d-flex flex-row-reverse gap-1 mt-3">
                            <button type="submit" class="btn btn-outline-primary">Update</button>
                            <button type="button" class="btn btn-outline-secondary" data-bs-dismiss="modal">Cancel</button>
                        </div>
                    </form>
                </div>

            </div>
        </div>
    </div>

    <!-- =========================
             DELETE Confirmation Modal
        ========================== -->
    <div class="modal fade" id="deleteLocationModal" tabindex="-1" aria-hidden="true">
        <div class="modal-dialog modal-dialog-centered" style="max-width: 450px;">
            <div class="modal-content p-4">
                <div class="modal-header border-0">
                    <h5 class="modal-title text-danger">Confirm Delete</h5>
                    <button type="button" class="btn-close" data-bs-dismiss="modal"></button>
                </div>

                <div class="modal-body">
                    <p>Are you sure you want to delete <strong id="delete-item-name"></strong>?</p>
                    <form id="deleteLocationForm" method="POST" action="#">
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

    <!-- =========================
             Inline Script
        ========================== -->
    <script>
        (function () {
            const citiesUrlTemplate = "{{ route('api.cities.by-country', ':id') }}";

            function setCityLoading(selectEl, isLoading) {
                selectEl.innerHTML = `<option value="">${isLoading ? 'Loading cities…' : '-- Select City --'}</option>`;
                selectEl.disabled = isLoading;
            }

            async function fetchCities(countryId, selectEl, preselectCityId = null) {
                if (!countryId) {
                    selectEl.innerHTML = '<option value="">-- Select City --</option>';
                    selectEl.disabled = true;
                    return;
                }
                setCityLoading(selectEl, true);

                try {
                    const url = citiesUrlTemplate.replace(':id', countryId);
                    const res = await fetch(url, { headers: { 'Accept': 'application/json' } });
                    if (!res.ok) throw new Error();

                    const cities = await res.json();
                    selectEl.innerHTML = '<option value="">-- Select City --</option>';
                    cities.forEach(c => {
                        const o = document.createElement('option');
                        o.value = c.id;
                        o.textContent = c.name;
                        if (preselectCityId && String(preselectCityId) === String(c.id)) o.selected = true;
                        selectEl.appendChild(o);
                    });
                    selectEl.disabled = false;
                } catch {
                    selectEl.innerHTML = '<option value="">(Failed to load cities)</option>';
                    selectEl.disabled = true;
                    if (window.Swal) {
                        Swal.fire({ toast: true, icon: 'error', title: 'Failed to load cities', timer: 2200, position: 'top-end', showConfirmButton: false });
                    }
                }
            }

            function wireCountryCity(modalEl) {
                const countrySelect = modalEl.querySelector('.country-select');
                const citySelect = modalEl.querySelector('.city-select');
                if (!countrySelect || !citySelect) return;

                countrySelect.addEventListener('change', () => {
                    fetchCities(countrySelect.value, citySelect);
                });
            }

            // ADD modal
            const addModalEl = document.getElementById('addLocationModal');
            addModalEl.addEventListener('show.bs.modal', () => {
                document.getElementById('addLocationForm').reset();
                const citySelect = addModalEl.querySelector('#add-city');
                citySelect.innerHTML = '<option value="">-- Select City --</option>';
                citySelect.disabled = true;
            });
            wireCountryCity(addModalEl);

            // Global click handler (Edit + Delete)
            document.addEventListener('click', e => {
                // EDIT
                const editBtn = e.target.closest('.editBtn');
                if (editBtn) {
                    const form = document.getElementById('editLocationForm');
                    form.setAttribute('action', editBtn.dataset.action);
                    document.getElementById('edit-name').value = editBtn.dataset.name;

                    const editCountry = document.getElementById('edit-country');
                    const editCity = document.getElementById('edit-city');
                    editCountry.value = editBtn.dataset.countryId || '';
                    fetchCities(editCountry.value, editCity, editBtn.dataset.cityId);

                    bootstrap.Modal.getOrCreateInstance(document.getElementById('editLocationModal')).show();
                    return;
                }

                // DELETE
                const deleteBtn = e.target.closest('.deleteBtn');
                if (deleteBtn) {
                    document.getElementById('delete-item-name').textContent = deleteBtn.dataset.name;
                    document.getElementById('deleteLocationForm').setAttribute('action', deleteBtn.dataset.action);
                    bootstrap.Modal.getOrCreateInstance(document.getElementById('deleteLocationModal')).show();
                }
            });

            wireCountryCity(document.getElementById('editLocationModal'));
        })();
    </script>
@endsection