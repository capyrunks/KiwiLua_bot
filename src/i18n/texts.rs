use super::Lang;

pub fn ready(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "🥝 Ready. Send a numeric AppID and I will return a ZIP with the Lua config.",
        Lang::Es => "🥝 Listo. Envíame un AppID numérico y te enviaré un ZIP con el config Lua.",
        Lang::Tr => "🥝 Hazırım. Sayısal bir AppID gönder; Lua config ZIP olarak gelsin.",
        Lang::It => "🥝 Pronto. Invia un AppID numerico e ti invierò uno ZIP con il config Lua.",
        Lang::Fr => {
            "🥝 Prêt. Envoyez un AppID numérique et je renverrai un ZIP avec le config Lua."
        }
        Lang::De => {
            "🥝 Bereit. Sende eine numerische AppID und ich schicke ein ZIP mit der Lua-Konfig."
        }
        Lang::Ru => "🥝 Готово. Отправьте числовой AppID, и я пришлю ZIP с Lua-конфигом.",
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
        Lang::En => "No config was found for this AppID.",
        Lang::Es => "No se encontró config para este AppID.",
        Lang::Tr => "Bu AppID için config bulunamadı.",
        Lang::It => "Nessun config trovato per questo AppID.",
        Lang::Fr => "Aucun config trouvé pour cet AppID.",
        Lang::De => "Keine Konfig für diese AppID gefunden.",
        Lang::Ru => "Для этого AppID конфиг не найден.",
    }
}

pub fn fetching_config(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Fetching the config from the cloud source...",
        Lang::Es => "Buscando el config en la fuente cloud...",
        Lang::Tr => "Config bulut kaynağından alınıyor...",
        Lang::It => "Recupero il config dalla sorgente cloud...",
        Lang::Fr => "Récupération du config depuis la source cloud...",
        Lang::De => "Konfig wird aus der Cloud-Quelle geladen...",
        Lang::Ru => "Забираю конфиг из облачного источника...",
    }
}

pub fn search_prompt(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Send a numeric Steam AppID.",
        Lang::Es => "Envía un AppID numérico de Steam.",
        Lang::Tr => "Sayısal bir Steam AppID gönderin.",
        Lang::It => "Invia un AppID Steam numerico.",
        Lang::Fr => "Envoyez un AppID Steam numérique.",
        Lang::De => "Sende eine numerische Steam-AppID.",
        Lang::Ru => "Отправьте числовой Steam AppID.",
    }
}

pub fn no_language_set() -> &'static str {
    "Please choose a language first.\n\nUse /start or /language."
}

pub fn app_id_only(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "For cloud mode I can only accept numeric Steam AppIDs, for example 730.",
        Lang::Es => "En modo cloud solo acepto AppIDs numéricos de Steam, por ejemplo 730.",
        Lang::Tr => "Bulut modunda yalnızca sayısal Steam AppID kabul ediyorum, örneğin 730.",
        Lang::It => "In modalità cloud accetto solo AppID Steam numerici, per esempio 730.",
        Lang::Fr => {
            "En mode cloud, seuls les AppID Steam numériques sont acceptés, par exemple 730."
        }
        Lang::De => "Im Cloud-Modus akzeptiere ich nur numerische Steam-AppIDs, zum Beispiel 730.",
        Lang::Ru => "В облачном режиме я принимаю только числовые Steam AppID, например 730.",
    }
}

pub fn source_unavailable(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "The config source is unavailable right now. Try again later or change the source URL.",
        Lang::Es => "La fuente de configs no está disponible ahora. Inténtalo más tarde o cambia la URL.",
        Lang::Tr => "Config kaynağı şu anda kullanılamıyor. Daha sonra deneyin veya kaynak URL'sini değiştirin.",
        Lang::It => "La sorgente dei config non è disponibile. Riprova più tardi o cambia URL.",
        Lang::Fr => "La source de configs est indisponible. Réessayez plus tard ou changez l'URL.",
        Lang::De => "Die Konfig-Quelle ist gerade nicht verfügbar. Später erneut versuchen oder URL ändern.",
        Lang::Ru => "Источник конфигов сейчас недоступен. Попробуйте позже или поменяйте URL источника.",
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
