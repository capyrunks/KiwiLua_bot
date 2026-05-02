use super::Lang;

pub fn ready(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "🥝 Ready. Send a game AppID or name and I will return a ZIP with .lua files.",
        Lang::Es => {
            "🥝 Listo. Envíame un AppID o nombre de juego y te enviaré un ZIP con archivos .lua."
        }
        Lang::Tr => "🥝 Hazırım. Bir oyun AppID'si veya adı gönder; .lua dosyalarını ZIP olarak göndereyim.",
        Lang::It => "🥝 Pronto. Inviami un AppID o nome del gioco e ti invierò uno ZIP con i file .lua.",
        Lang::Fr => "🥝 Prêt. Envoyez un AppID ou un nom de jeu et je renverrai un ZIP avec les fichiers .lua.",
        Lang::De => "🥝 Bereit. Sende eine AppID oder einen Spielnamen und ich schicke ein ZIP mit .lua-Dateien.",
        Lang::Ru => "🥝 Готово. Отправьте AppID или название игры, и я пришлю ZIP с .lua файлами.",
    }
}

pub fn choose_language(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Choose your language:",
        Lang::Es => "Elige tu idioma:",
        Lang::Tr => "Dilinizi seçin:",
        Lang::It => "Scegli la lingua:",
        Lang::Fr => "Choisissez votre langue :",
        Lang::De => "Sprache auswählen:",
        Lang::Ru => "Выберите язык:",
    }
}

pub fn choose_language_initial() -> &'static str {
    "Welcome to KiwiLua Bot.\n\nChoose your language:"
}

pub fn language_set(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Language set to English.",
        Lang::Es => "Idioma establecido: Español.",
        Lang::Tr => "Dil Türkçe olarak ayarlandı.",
        Lang::It => "Lingua impostata: Italiano.",
        Lang::Fr => "Langue définie : Français.",
        Lang::De => "Sprache eingestellt: Deutsch.",
        Lang::Ru => "Язык установлен: Русский.",
    }
}

pub fn help_text(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Developer contact: @KiwiLua\nResponse time: 2-4 days.",
        Lang::Es => "Contacto del desarrollador: @KiwiLua\nTiempo de respuesta: 2-4 días.",
        Lang::Tr => "Geliştirici iletişimi: @KiwiLua\nYanıt süresi: 2-4 gün.",
        Lang::It => "Contatto sviluppatore: @KiwiLua\nTempo di risposta: 2-4 giorni.",
        Lang::Fr => "Contact développeur : @KiwiLua\nDélai de réponse : 2-4 jours.",
        Lang::De => "Entwicklerkontakt: @KiwiLua\nAntwortzeit: 2-4 Tage.",
        Lang::Ru => "Контакт разработчика: @KiwiLua\nВремя ответа: 2-4 дня.",
    }
}

pub fn not_found(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "No .lua files were found for this query. Try an AppID like 730.",
        Lang::Es => "No se encontraron archivos .lua. Prueba con un AppID como 730.",
        Lang::Tr => ".lua dosyası bulunamadı. 730 gibi bir AppID deneyin.",
        Lang::It => "Nessun file .lua trovato. Prova un AppID come 730.",
        Lang::Fr => "Aucun fichier .lua trouvé. Essayez un AppID comme 730.",
        Lang::De => "Keine .lua-Dateien gefunden. Versuche eine AppID wie 730.",
        Lang::Ru => ".lua файлы не найдены. Попробуйте AppID, например 730.",
    }
}

pub fn sending_files(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Packing files in memory and sending the archive...",
        Lang::Es => "Empaquetando archivos en memoria y enviando el archivo...",
        Lang::Tr => "Dosyalar bellekte paketleniyor ve arşiv gönderiliyor...",
        Lang::It => "Impacchetto i file in memoria e invio l'archivio...",
        Lang::Fr => "Compression des fichiers en mémoire et envoi de l'archive...",
        Lang::De => "Dateien werden im Speicher gepackt und das Archiv wird gesendet...",
        Lang::Ru => "Упаковываю файлы в оперативной памяти и отправляю архив...",
    }
}

pub fn search_prompt(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Send a game AppID or name to search for .lua files.",
        Lang::Es => "Envía un AppID o nombre de juego para buscar archivos .lua.",
        Lang::Tr => ".lua dosyalarını aramak için bir oyun AppID'si veya adı gönderin.",
        Lang::It => "Invia un AppID o il nome del gioco per cercare file .lua.",
        Lang::Fr => "Envoyez un AppID ou un nom de jeu pour rechercher des fichiers .lua.",
        Lang::De => "Sende eine AppID oder einen Spielnamen, um .lua-Dateien zu suchen.",
        Lang::Ru => "Отправьте AppID или название игры для поиска .lua файлов.",
    }
}

pub fn no_language_set() -> &'static str {
    "Please choose a language first.\n\nUse /start or /language."
}

pub fn query_too_long(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "The query is too long. Send a shorter AppID or game name.",
        Lang::Es => "La consulta es demasiado larga. Envía un AppID o nombre más corto.",
        Lang::Tr => "Sorgu çok uzun. Daha kısa bir AppID veya oyun adı gönderin.",
        Lang::It => "La ricerca è troppo lunga. Invia un AppID o nome più breve.",
        Lang::Fr => "La recherche est trop longue. Envoyez un AppID ou un nom plus court.",
        Lang::De => "Die Anfrage ist zu lang. Sende eine kürzere AppID oder einen kürzeren Namen.",
        Lang::Ru => "Запрос слишком длинный. Отправьте более короткий AppID или название игры.",
    }
}

pub fn packing_error(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Could not create the ZIP archive. Please try again later.",
        Lang::Es => "No se pudo crear el archivo ZIP. Inténtalo más tarde.",
        Lang::Tr => "ZIP arşivi oluşturulamadı. Lütfen daha sonra tekrar deneyin.",
        Lang::It => "Impossibile creare l'archivio ZIP. Riprova più tardi.",
        Lang::Fr => "Impossible de créer l'archive ZIP. Réessayez plus tard.",
        Lang::De => "Das ZIP-Archiv konnte nicht erstellt werden. Bitte später erneut versuchen.",
        Lang::Ru => "Не удалось создать ZIP архив. Попробуйте позже.",
    }
}
