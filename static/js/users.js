const UserManager = {
    allUsers: [],
    filteredUsers: [],
    currentPage: 1,
    rowsPerPage: 5,

    init() {
        this.form = document.getElementById('userForm');
        this.tableBody = document.getElementById('userTableBody');
        this.searchInput = document.getElementById('searchInput');
        this.dateFrom = document.getElementById('dateFrom');
        this.dateTo = document.getElementById('dateTo');
        this.btnClearFilters = document.getElementById('btnClearFilters');

        // Modales
        this.userModal = new bootstrap.Modal(document.getElementById('userModal'));
        this.deleteModal = new bootstrap.Modal(document.getElementById('deleteModal'));

        // Elementos de la imagen
        this.imagenInput = document.getElementById('imagenInput');
        this.previewContainer = document.getElementById('previewContainer');
        this.previewImg = document.getElementById('previewImg');
        this.fileError = document.getElementById('fileError');

        // Paginación
        this.btnFirst = document.getElementById('btnFirst');
        this.btnPrev = document.getElementById('btnPrev');
        this.btnNext = document.getElementById('btnNext');
        this.btnLast = document.getElementById('btnLast');
        this.pageIndicator = document.getElementById('currentPageIndicator');

        this.bindEvents();
        this.loadCatalogs(); // Opcional: Cargar Selects
        this.loadUsers();
    },

    bindEvents() {
        this.form.addEventListener('submit', (e) => this.handleSubmit(e));

        // Filtros (Texto y Fechas)
        [this.searchInput, this.dateFrom, this.dateTo].forEach(el => {
            if (el) el.addEventListener('input', () => this.applyFilters());
        });

        if (this.btnClearFilters) {
            this.btnClearFilters.addEventListener('click', () => this.clearFilters());
        }

        // Previsualización de imagen
        if (this.imagenInput) {
            this.imagenInput.addEventListener('change', (e) => this.handleImagePreview(e));
        }

        // Paginación
        if (this.btnFirst) this.btnFirst.onclick = () => { this.currentPage = 1; this.renderTableWithPagination(); };
        if (this.btnPrev) this.btnPrev.onclick = () => { if (this.currentPage > 1) { this.currentPage--; this.renderTableWithPagination(); } };
        if (this.btnNext) this.btnNext.onclick = () => {
            const maxPage = Math.ceil(this.filteredUsers.length / this.rowsPerPage);
            if (this.currentPage < maxPage) { this.currentPage++; this.renderTableWithPagination(); }
        };
        if (this.btnLast) this.btnLast.onclick = () => {
            this.currentPage = Math.ceil(this.filteredUsers.length / this.rowsPerPage) || 1;
            this.renderTableWithPagination();
        };
    },

    handleImagePreview(e) {
        const file = e.target.files[0];
        this.fileError.classList.add('d-none');

        if (file) {
            // Validar extensión
            const allowedExtensions = ['image/jpeg', 'image/png', 'image/webp'];
            if (!allowedExtensions.includes(file.type)) {
                this.fileError.classList.remove('d-none');
                this.imagenInput.value = ''; // Limpiar input
                this.previewContainer.classList.add('d-none');
                return;
            }

            const reader = new FileReader();
            reader.onload = (event) => {
                this.previewImg.src = event.target.result;
                this.previewContainer.classList.remove('d-none');
            };
            reader.readAsDataURL(file);
        } else {
            this.previewContainer.classList.add('d-none');
        }
    },

    applyFilters() {
        const term = this.searchInput.value.toLowerCase().trim();
        const start = this.dateFrom.value;
        const end = this.dateTo.value;

        this.filteredUsers = this.allUsers.filter(u => {
            // 1. Filtro de Texto (Nombre o Apellidos)
            const fullName = `${u.nombre || ''} ${u.apellido_p || ''} ${u.apellido_m || ''}`.toLowerCase();
            const email = (u.str_correo || "").toLowerCase();
            const matchesTerm = term === "" || fullName.includes(term) || email.includes(term);

            // 2. Filtro de Nacimiento
            const userDate = (u.fecha_nacimiento || "").substring(0, 10);
            const matchesStart = !start || userDate >= start;
            const matchesEnd = !end || userDate <= end;

            return matchesTerm && matchesStart && matchesEnd;
        });

        this.currentPage = 1;
        this.renderTableWithPagination();
    },

    renderTableWithPagination() {
        const total = this.filteredUsers.length;
        const maxPage = Math.ceil(total / this.rowsPerPage) || 1;

        if (this.currentPage > maxPage) this.currentPage = maxPage;
        if (this.currentPage < 1) this.currentPage = 1;

        const startIdx = (this.currentPage - 1) * this.rowsPerPage;
        const pagedData = this.filteredUsers.slice(startIdx, startIdx + this.rowsPerPage);

        this.renderTable(pagedData);

        if (this.pageIndicator) {
            this.pageIndicator.innerText = `Página ${this.currentPage} de ${maxPage} (${total} lectores)`;
        }

        const isFirst = this.currentPage === 1;
        const isLast = this.currentPage === maxPage || total === 0;

        if (this.btnFirst) this.btnFirst.parentElement.classList.toggle('disabled', isFirst);
        if (this.btnPrev) this.btnPrev.parentElement.classList.toggle('disabled', isFirst);
        if (this.btnNext) this.btnNext.parentElement.classList.toggle('disabled', isLast);
        if (this.btnLast) this.btnLast.parentElement.classList.toggle('disabled', isLast);
    },

    renderTable(users) {
        console.log("🎨 Renderizando tabla con", users.length, "usuarios");

        if (users.length > 0) {
            this.tableBody.innerHTML = users.map((u, index) => {
                console.log(`   Usuario ${index + 1}:`, {
                    id: u.id,
                    nombre: u.nombre,
                    apellido_p: u.apellidoP,  // ✅ CORRECTO: usa apellidoP (camelCase)
                    apellido_m: u.apellidoM,  // ✅ CORRECTO: usa apellidoM (camelCase)
                    correo: u.strCorreo,       // ✅ CORRECTO: usa strCorreo
                    imagen: u.strImagenPath    // ✅ CORRECTO: usa strImagenPath
                });

                // Construcción del nombre completo
                const fullName = `${u.nombre || ''} ${u.apellidoP || ''} ${u.apellidoM || ''}`.trim();

                const defaultAvatar = 'static/images/user_perfil.png';
                const avatarUrl = u.strImagenPath ? `/${u.strImagenPath.replace(/\\/g, '/')}` : defaultAvatar;

                // Lógica de permisos de botones
                let btnHtml = '';
                if (typeof PERMISOS_MODULO !== 'undefined') {
                    if (PERMISOS_MODULO.canEdit) {
                        btnHtml += `<button class="action-btn edit-btn" onclick="UserManager.openModal(${u.id})" title="Editar Ficha"><i class="fas fa-pen-fancy"></i></button>`;
                    }
                    if (PERMISOS_MODULO.canDelete) {
                        btnHtml += `<button class="action-btn delete-btn" onclick="UserManager.confirmDelete(${u.id})" title="Retirar Lector"><i class="fas fa-trash"></i></button>`;
                    }
                } else {
                    btnHtml = `
                <button class="action-btn edit-btn" onclick="UserManager.openModal(${u.id})"><i class="fas fa-pen-fancy"></i></button>
                <button class="action-btn delete-btn" onclick="UserManager.confirmDelete(${u.id})"><i class="fas fa-trash"></i></button>`;
                }

                return `
        <tr>
            <td><img src="${avatarUrl}" class="avatar-table" alt="Foto" onerror="this.onerror=null; this.src='${defaultAvatar}';"></td>
            <td><strong>${fullName}</strong></td>
            <td>${u.strCorreo || 'Sin correo'}</td>
            <td>${u.idSexo === 1 ? 'Masc' : u.idSexo === 2 ? 'Fem' : 'Otro'}</td>
            <td>${u.strNumeroCelular || 'N/A'}</td>
            <td><span class="badge bg-secondary">Perfil ${u.idPerfil || 'N/A'}</span></td>
            <td><span class="badge ${u.idEstadoUsuario === 1 ? 'bg-success' : 'bg-danger'}">${u.idEstadoUsuario === 1 ? 'Activo' : 'Inactivo'}</span></td>
            <td class="text-center">${btnHtml}</td>
        </tr>
    `}).join('');

            console.log("✅ Tabla renderizada exitosamente");
        } else {
            console.warn("⚠️ No hay usuarios para mostrar");
            this.tableBody.innerHTML = '<tr><td colspan="8" class="text-center py-4 text-muted italic">No se encontraron lectores en el archivo.</td></tr>';
        }
    },

    async loadUsers() {
        this.renderSkeleton();
        try {
            console.log("⏳ Solicitando datos al backend...");
            const res = await fetch('/api/usuarios');

            // 1. Verificamos si el servidor respondió correctamente
            if (!res.ok) {
                const errorText = await res.text();
                throw new Error(`El servidor respondió con código ${res.status}: ${errorText}`);
            }

            const data = await res.json();

            // 2. 🔍 DIAGNÓSTICO COMPLETO
            console.log("📦 Datos recibidos de Rust:");
            console.log("   - Tipo de dato:", typeof data);
            console.log("   - Es un Array:", Array.isArray(data));
            console.log("   - Longitud:", data.length);
            console.log("   - Contenido completo:", JSON.stringify(data, null, 2));

            // 3. Verificar si hay datos
            if (!Array.isArray(data)) {
                throw new Error(`Se esperaba un array pero se recibió: ${typeof data}`);
            }

            if (data.length === 0) {
                console.warn("⚠️ El backend devolvió un array vacío");
            } else {
                console.log(`✅ Se recibieron ${data.length} usuarios correctamente`);
                console.log("   - Primer usuario:", data[0]);
            }

            this.allUsers = data;
            this.applyFilters();

        } catch (e) {
            console.error("🚨 ERROR FATAL AL CARGAR USUARIOS:", e);

            if (typeof this.showToast === 'function') {
                this.showToast("Error al procesar los datos de los lectores", 'error');
            }

            this.tableBody.innerHTML = `
            <tr>
                <td colspan="8" class="text-center text-danger py-4">
                    <b>Error detectado:</b> ${e.message}<br>
                    <small>Revisa la consola del navegador (F12) para más detalles.</small>
                </td>
            </tr>`;
        }
    },

    renderSkeleton() {
        const skeletonRow = `<tr><td colspan="8"><div class="placeholder-glow"><span class="placeholder col-12 bg-light" style="height: 30px;"></span></div></td></tr>`;
        this.tableBody.innerHTML = skeletonRow.repeat(8);
    },

    clearFilters() {
        this.searchInput.value = "";
        this.dateFrom.value = "";
        this.dateTo.value = "";
        this.applyFilters();
    },

    async openModal(id = null) {
        this.form.reset();
        this.clearErrors();

        // Resetear imagen visual
        this.previewContainer.classList.add('d-none');
        this.previewImg.src = "";
        this.fileError.classList.add('d-none');

        const modalTitle = document.getElementById('modalTitle');
        const submitBtn = this.form.querySelector('button[type="submit"]');
        const passwordLabel = document.querySelector('#passwordGroup label');

        document.getElementById('userId').value = id || "";
        modalTitle.innerText = id ? 'Editar Ficha de Lector' : 'Nueva Ficha de Lector';
        submitBtn.innerText = id ? 'Actualizar Ficha' : 'Guardar en Archivo';

        if (id) {
            try {
                const res = await fetch(`/api/usuarios/${id}`);
                const u = await res.json();

                console.log("📝 Datos recibidos del backend:", u); // DEBUG

                // ✅ MAPEO CORRECTO: Usar los IDs de los inputs del HTML
                document.getElementById('name').value = u.nombre || "";
                document.getElementById('ap_paterno').value = u.apellidoP || "";  // ✅ CAMBIO: era apellido_p
                document.getElementById('ap_materno').value = u.apellidoM || "";  // ✅ CAMBIO: era apellido_m
                document.getElementById('emailInput').value = u.strCorreo || "";
                document.getElementById('phoneInput').value = u.strNumeroCelular || "";
                document.getElementById('birthdateInput').value = (u.fechaNacimiento || "").substring(0, 10);

                // Selects (Catálogos)
                document.getElementById('sexoSelect').value = u.idSexo || "";        // ✅ CAMBIO: era id_sexo
                document.getElementById('deptoSelect').value = u.idPerfil || "";      // ✅ CAMBIO: era id_perfil
                document.getElementById('estadoSelect').value = u.idEstadoUsuario || ""; // ✅ CAMBIO: era id_estado_usuario

                // Imagen previa si existe
                if (u.strImagenPath) {  // ✅ CAMBIO: era str_imagen_path
                    this.previewImg.src = `/${u.strImagenPath.replace(/\\/g, '/')}`;
                    this.previewContainer.classList.remove('d-none');
                }

                passwordLabel.innerText = "Nueva Contraseña (opcional)";
            } catch (e) {
                console.error("❌ Error al cargar datos del usuario:", e);
                this.showToast("No se pudo recuperar la ficha", 'error');
            }
        } else {
            passwordLabel.innerText = "Contraseña";
        }
        this.userModal.show();
    },

    async handleSubmit(e) {
        e.preventDefault();
        this.clearErrors();

        const submitBtn = this.form.querySelector('button[type="submit"]');
        submitBtn.disabled = true;
        submitBtn.innerHTML = '<i class="fas fa-spinner fa-spin"></i> Guardando...';

        // 1. Extraemos los datos crudos del formulario
        const rawFormData = new FormData(this.form);
        const backendFormData = new FormData();

        // 2. Mapeamos las llaves del HTML hacia el camelCase que espera Rust
        const id = rawFormData.get('id');
        if (id) backendFormData.append('id', id);

        backendFormData.append('nombre', rawFormData.get('Nombre'));
        backendFormData.append('apellidoP', rawFormData.get('ApellidoP'));      // ✅ camelCase
        backendFormData.append('apellidoM', rawFormData.get('ApellidoM'));      // ✅ camelCase
        backendFormData.append('strCorreo', rawFormData.get('strCorreo'));
        backendFormData.append('strNumeroCelular', rawFormData.get('strNumeroCelular'));
        backendFormData.append('fechaNacimiento', rawFormData.get('FechaNacimiento'));
        backendFormData.append('idSexo', rawFormData.get('idSexo'));            // ✅ camelCase
        backendFormData.append('idPerfil', rawFormData.get('idPerfil'));        // ✅ camelCase
        backendFormData.append('idEstadoUsuario', rawFormData.get('idEstadoUsuario')); // ✅ camelCase

        // Controlamos la contraseña (si está vacía en edición, no la mandamos)
        const pwd = rawFormData.get('strPwd');
        if (pwd) backendFormData.append('strPwd', pwd);

        // Controlamos la imagen
        const file = rawFormData.get('imagenInput');
        if (file && file.size > 0) {
            backendFormData.append('imagenInput', file);
        }

        console.log("📤 Datos que se enviarán:");
        for (let [key, value] of backendFormData.entries()) {
            console.log(`   ${key}:`, value);
        }

        try {
            const res = await fetch('/api/usuarios', {
                method: id ? 'PUT' : 'POST',
                body: backendFormData
            });

            const result = await res.json();

            if (result.success) {
                this.showToast(result.msg, 'success');
                this.userModal.hide();
                await this.loadUsers();
            } else {
                if (result.errors && result.errors.email) {
                    document.getElementById('error_email').innerText = result.errors.email;
                }
                this.showToast(result.msg || "Verifica los datos de la ficha", 'error');
            }
        } catch (e) {
            console.error("❌ Error en submit:", e);
            this.showToast("Error de conexión con el servidor", 'error');
        } finally {
            submitBtn.disabled = false;
            submitBtn.innerText = id ? 'Actualizar Ficha' : 'Guardar en Archivo';
        }
    },

    confirmDelete(id) {
        this.userToDeleteId = id;
        this.deleteModal.show();
    },

    async executeDelete() {
        const id = this.userToDeleteId;
        const btn = document.getElementById('confirmDeleteBtn');
        btn.disabled = true;
        try {
            const res = await fetch(`/api/usuarios/${id}`, {
                method: 'DELETE'
            });
            const result = await res.json();
            if (result.success) {
                this.showToast("Lector retirado del archivo", 'warning');
                this.deleteModal.hide();
                await this.loadUsers();
            } else {
                this.showToast(result.msg, 'error');
            }
        } catch (e) {
            this.showToast("Error al retirar la ficha", 'error');
        } finally {
            btn.disabled = false;
        }
    },

    clearErrors() {
        const errEmail = document.getElementById('error_email');
        if (errEmail) errEmail.innerText = "";
    },

    showToast(msg, type = 'success') {
        let container = document.querySelector('.toast-container');
        if (!container) {
            container = document.createElement('div');
            container.className = 'toast-container position-fixed bottom-0 end-0 p-3';
            container.style.zIndex = "2000";
            document.body.appendChild(container);
        }

        const toast = document.createElement('div');
        toast.className = `toast align-items-center text-white bg-${type === 'error' ? 'danger' : type === 'warning' ? 'warning' : 'dark'} border-0 show`;
        toast.innerHTML = `
            <div class="d-flex">
                <div class="toast-body"><i class="fas fa-info-circle me-2"></i> ${msg}</div>
                <button type="button" class="btn-close btn-close-white me-2 m-auto" data-bs-dismiss="toast"></button>
            </div>`;
        container.appendChild(toast);
        setTimeout(() => toast.remove(), 4000);
    },

    async loadCatalogs() {
        const sexoSelect = document.getElementById('sexoSelect');
        if (sexoSelect && sexoSelect.options.length <= 1) {
            sexoSelect.innerHTML += `<option value="1">Masculino</option><option value="2">Femenino</option><option value="3">No Binario</option>`;
        }

        const deptoSelect = document.getElementById('deptoSelect');
        if (deptoSelect) {
            try {
                const res = await fetch('/api/perfil');
                if (res.ok) {
                    const perfiles = await res.json();
                    console.log("✅ Perfiles cargados desde la BD:", perfiles);
                    
                    // Limpiar opciones existentes
                    deptoSelect.innerHTML = '';
                    
                    // Construir cadena HTML con todas las opciones
                    let optionsHtml = '';
                    perfiles.forEach(perfil => {
                        console.log("Perfil individual:", perfil); // DEBUG
                        optionsHtml += `<option value="${perfil.id}">${perfil.strNombrePerfil}</option>`;
                    });
                    
                    deptoSelect.innerHTML = optionsHtml;
                } else {
                    console.warn("⚠️ No se pudieron cargar los perfiles, usando defaults");
                    deptoSelect.innerHTML = `<option value="1">Administrador</option><option value="2">Operador</option>`;
                }
            } catch (e) {
                console.error("❌ Error al cargar perfiles:", e);
                deptoSelect.innerHTML = `<option value="1">Administrador</option><option value="2">Operador</option>`;
            }
        }

        const estadoSelect = document.getElementById('estadoSelect');
        if (estadoSelect && estadoSelect.options.length <= 1) {
            estadoSelect.innerHTML += `<option value="1">Activo</option><option value="2">Inactivo</option>`;
        }
    }
};

document.addEventListener('DOMContentLoaded', () => {
    UserManager.init();

    // Asignar el botón de confirmar borrado (ya que el modal lo reconstruye Bootstrap)
    document.getElementById('confirmDeleteBtn').addEventListener('click', () => UserManager.executeDelete());
});
